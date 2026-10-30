//! Backward-compatible effect selectors. A supplied instance ID is always authoritative.
use filmcraft_project::{ClipId, EffectInstance};
use serde_json::{Value, json};

use crate::{EffectSel, EngineError, Result, Session, commands::bad};

pub(crate) fn index(effects: &[EffectInstance], selector: &Value) -> Option<usize> {
    match selector {
        Value::Number(value) => usize::try_from(value.as_u64()?).ok().filter(|i| effects.get(*i).is_some()),
        Value::String(effect) => effects.iter().position(|e| e.effect == *effect),
        Value::Object(fields) if fields.len() == 1 => EffectInstance::instance_index(effects, fields.get("instanceId")?.as_u64()?),
        _ => None,
    }
}

pub(crate) fn resolve_mut<'a>(effects: &'a mut [EffectInstance], selector: &Value) -> Option<&'a mut EffectInstance> {
    let index = index(effects, selector)?;
    effects.get_mut(index)
}

pub(crate) fn clip_index(s: &Session, clip: ClipId, p: &Value, command: &str) -> Result<usize> {
    if p.get("effect").is_some() && p.get("index").is_some() {
        return Err(bad(command, "specify effect or index, not both"));
    }
    if p.get("index").is_some_and(|v| v.as_u64().is_none()) {
        return Err(bad(command, "index must be a nonnegative integer"));
    }
    let effects = s.active_sequence().and_then(|q| q.find_item(clip)).map(|(_, it)| &it.effects).ok_or_else(|| bad(command, "clip does not exist"))?;
    index(effects, p.get("effect").or_else(|| p.get("index")).unwrap_or(&json!(0)))
        .ok_or_else(|| bad(command, "effect does not exist or its instance identity is ambiguous"))
}

pub(crate) fn duplicate(s: &mut Session, p: &Value) -> Result<Value> {
    let clip = p.get("clip").and_then(Value::as_u64).map(ClipId).ok_or_else(|| bad("effects.duplicate", "need clip"))?;
    let at = clip_index(s, clip, p, "effects.duplicate")?;
    let effect = s
        .active_sequence()
        .and_then(|q| q.find_item(clip))
        .and_then(|(_, it)| it.effects.get(at))
        .ok_or_else(|| bad("effects.duplicate", "effect no longer exists"))?;
    if effect.def().is_some_and(|d| d.intrinsic) || filmcraft_project::graphic::is_layer(effect) {
        return Err(bad("effects.duplicate", "fixed attributes cannot be duplicated; use graphic layer commands for graphics"));
    }
    let copy = effect.fresh_copy();
    let target = at.checked_add(1).ok_or_else(|| bad("effects.duplicate", "effect position is out of range"))?;
    s.edit_sequence("Duplicate Effect", |q, _, _| {
        let (_, it) = q.find_item_mut(clip).ok_or(filmcraft_edit::EditError::NoItem(clip))?;
        if target > it.effects.len() {
            return Err(bad("effects.duplicate", "effect position no longer exists"));
        }
        it.effects.insert(target, copy);
        Ok(())
    })?;
    let instance = s
        .active_sequence()
        .and_then(|q| q.find_item(clip))
        .and_then(|(_, it)| it.effects.get(target))
        .map(|e| e.instance_id)
        .ok_or_else(|| bad("effects.duplicate", "duplicate identity was not attached"))?;
    Ok(json!({"clip":clip.0,"instanceId":instance}))
}

pub(crate) fn reorder(s: &mut Session, p: &Value) -> Result<Value> {
    let clip = p.get("clip").and_then(Value::as_u64).map(ClipId).ok_or_else(|| bad("effects.reorder", "need clip"))?;
    let from = clip_index(s, clip, p, "effects.reorder")?;
    let to = p
        .get("to")
        .and_then(Value::as_u64)
        .and_then(|v| usize::try_from(v).ok())
        .ok_or_else(|| bad("effects.reorder", "to must be an added-effect position"))?;
    let it = s.active_sequence().and_then(|q| q.find_item(clip)).map(|(_, it)| it).ok_or(EngineError::NoSequence)?;
    let movable = |e: &EffectInstance| !e.def().is_some_and(|d| d.intrinsic) && !filmcraft_project::graphic::is_layer(e);
    if !it.effects.get(from).is_some_and(movable) {
        return Err(bad("effects.reorder", "only added effects can be reordered here"));
    }
    let dest = it
        .effects
        .iter()
        .enumerate()
        .filter(|(_, e)| movable(e))
        .nth(to)
        .map(|(i, _)| i)
        .ok_or_else(|| bad("effects.reorder", "added-effect position is out of range"))?;
    let selected =
        s.state.selected_effect.as_ref().filter(|e| e.clip == clip).and_then(|e| e.index(&it.effects)).and_then(|i| EffectSel::of(clip, &it.effects, i));
    if from != dest {
        s.edit_sequence("Reorder Effect", |q, _, _| {
            let (_, it) = q.find_item_mut(clip).ok_or(filmcraft_edit::EditError::NoItem(clip))?;
            if from >= it.effects.len() || dest >= it.effects.len() {
                return Err(bad("effects.reorder", "effect position no longer exists"));
            }
            let effect = it.effects.remove(from);
            it.effects.insert(dest, effect);
            Ok(())
        })?;
    }
    if selected.is_some() {
        s.state.selected_effect = selected;
    }
    s.fix_state();
    Ok(json!({"clip":clip.0,"to":to}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn fixture() -> (Session, ClipId) {
        let mut s = Session::default();
        s.execute("file.openDemoProject", json!({})).unwrap();
        let clip = s.active_sequence().unwrap().video_tracks[0].items[0].id;
        s.execute("effects.apply", json!({"clips":[clip.0],"effect":"gaussian_blur"})).unwrap();
        (s, clip)
    }

    fn blur(s: &Session, clip: ClipId) -> u64 {
        s.active_sequence().unwrap().find_item(clip).unwrap().1.effects.iter().find(|e| e.effect == "gaussian_blur").unwrap().instance_id
    }

    #[test]
    fn duplicate_reorder_remove_and_history_keep_instances_and_selections_distinct() {
        let (mut s, clip) = fixture();
        let first = blur(&s, clip);
        let copy = s.execute("effects.duplicate", json!({"clip":clip.0,"effect":{"instanceId":first}})).unwrap();
        let second = copy["instanceId"].as_u64().unwrap();
        assert_ne!(first, second);
        s.execute("effects.setParam", json!({"clip":clip.0,"effect":{"instanceId":second},"param":"blurriness","value":37.0})).unwrap();
        s.execute("effects.select", json!({"clip":clip.0,"effect":{"instanceId":second}})).unwrap();
        s.execute("masks.add", json!({"clip":clip.0,"effect":{"instanceId":second},"shape":"ellipse"})).unwrap();
        s.execute("effects.reorder", json!({"clip":clip.0,"effect":{"instanceId":second},"to":0})).unwrap();
        let it = s.active_sequence().unwrap().find_item(clip).unwrap().1;
        assert_eq!(it.effects[0].instance_id, second);
        assert_eq!(it.effects[0].f64_at("blurriness", filmcraft_time::Tick::ZERO), 37.0);
        assert_eq!(s.state.selected_mask.unwrap().effect_id, Some(second));
        assert_eq!(s.state.selected_mask.unwrap().effect, 0);
        s.execute("effects.remove", json!({"clip":clip.0,"effect":{"instanceId":first}})).unwrap();
        let before_remove = s.project.clone();
        s.execute("effects.remove", json!({"clip":clip.0,"effect":{"instanceId":second}})).unwrap();
        assert!(s.state.selected_mask.is_none());
        s.execute("edit.undo", json!({})).unwrap();
        assert_eq!(*s.project, *before_remove);
        s.execute("edit.redo", json!({})).unwrap();
        assert!(s.execute("effects.setParam", json!({"clip":clip.0,"effect":{"instanceId":second},"param":"blurriness","value":5.0})).is_err());
    }

    #[test]
    fn undo_branch_and_persistence_never_recycle_retired_effect_ids() {
        let (mut s, clip) = fixture();
        let first = blur(&s, clip);
        s.execute("edit.undo", json!({})).unwrap();
        let saved = s.project_for_persistence();
        assert!(saved.next_effect_instance_id > first);
        let loaded = filmcraft_format::decode(&filmcraft_format::encode(&saved, false)).unwrap();
        assert_eq!(loaded.project.next_effect_instance_id, saved.next_effect_instance_id);
        s.execute("effects.apply", json!({"clips":[clip.0],"effect":"gaussian_blur"})).unwrap();
        assert!(blur(&s, clip) > first);
    }

    #[test]
    fn authoritative_stale_or_hostile_selectors_never_fall_back_or_edit() {
        let (mut s, clip) = fixture();
        let before = s.project.clone();
        let undo = s.history.undo.len();
        for selector in [json!({"instanceId":0}), json!({"instanceId":u64::MAX}), json!({"instanceId":blur(&s, clip),"index":0}), json!(-1)] {
            for command in ["effects.select", "effects.remove", "effects.toggleEnabled", "effects.reset", "effects.duplicate", "effects.reorder"] {
                assert!(s.execute(command, json!({"clip":clip.0,"effect":selector,"to":0})).is_err(), "{command}: {selector}");
                assert!(Arc::ptr_eq(&s.project, &before));
                assert_eq!(s.history.undo.len(), undo);
            }
        }
    }

    #[test]
    fn project_generation_changes_on_replacement_even_when_demo_ids_repeat() {
        let (mut s, clip) = fixture();
        let generation = s.project_generation();
        s.execute("effects.toggleEnabled", json!({"clip":clip.0,"effect":{"instanceId":blur(&s, clip)}})).unwrap();
        assert_eq!(s.project_generation(), generation);
        s.execute("file.openDemoProject", json!({})).unwrap();
        assert_ne!(s.project_generation(), generation);
        assert_eq!(s.active_sequence().unwrap().video_tracks[0].items[0].id, clip);
    }
}
