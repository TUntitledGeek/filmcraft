//! Stable effect-instance metadata. IDs are scoped by their owning clip/strip/source graphic.
//! Constructors leave ID zero until attached; snapshots and clip copies retain assigned IDs.
use std::{collections::BTreeSet, sync::Arc};

use crate::{EffectInstance, ItemKind, Project, Sequence};

pub(crate) fn initial_counter() -> u64 {
    1
}

fn visit_sequence(sequence: &Sequence, f: &mut impl FnMut(&[EffectInstance]) -> Result<(), String>) -> Result<(), String> {
    f(&sequence.master_effects)?;
    for track in sequence.all_tracks().chain(&sequence.submix_tracks) {
        f(&track.effects)?;
        for clip in &track.items {
            f(&clip.effects)?;
        }
        for transition in &track.transitions {
            f(std::slice::from_ref(&transition.effect))?;
        }
    }
    Ok(())
}

fn needs_ids(effects: &[EffectInstance]) -> bool {
    let mut seen = BTreeSet::new();
    effects.iter().any(|e| e.instance_id == 0 || !seen.insert(e.instance_id))
}

fn assign(effects: &mut [EffectInstance], next: &mut u64, changed: &mut usize) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for effect in effects {
        if effect.instance_id != 0 && seen.insert(effect.instance_id) {
            continue;
        }
        let after = next.checked_add(1).ok_or("Effect instance identity space is exhausted")?;
        effect.instance_id = *next;
        *next = after;
        *changed = changed.saturating_add(1);
        seen.insert(effect.instance_id);
    }
    Ok(())
}

impl Project {
    /// Assign IDs without changing effect values, keyframes, masks, order or graphic-layer IDs.
    /// The session's allocation floor stays outside undo snapshots, so an abandoned redo branch
    /// cannot cause an old selection to attach to a newly created effect. Loaded legacy files
    /// may repair missing/duplicate metadata; an unexpected duplicate in a live edit is refused.
    /// All validation/capacity checks precede mutation. Valid unrelated sequences stay shared.
    pub fn assign_effect_instance_ids(&mut self, allocation_floor: &mut u64, repair_duplicates: bool) -> Result<usize, String> {
        let mut largest = 0_u64;
        let mut missing = 0_u64;
        let mut check = |effects: &[EffectInstance]| {
            let mut seen = BTreeSet::new();
            for effect in effects {
                largest = largest.max(effect.instance_id);
                let duplicate = effect.instance_id != 0 && !seen.insert(effect.instance_id);
                if duplicate && !repair_duplicates {
                    return Err("Duplicate effect instance identity in one owner; create a fresh effect identity when copying an effect".into());
                }
                if effect.instance_id == 0 || duplicate {
                    missing = missing.checked_add(1).ok_or("Too many effect instances")?;
                }
            }
            Ok(())
        };
        for item in self.items.values() {
            if let ItemKind::Sequence(sequence) = &item.kind {
                visit_sequence(sequence, &mut check)?;
            }
        }
        for source in self.source_graphics.values() {
            check(&source.layers)?;
        }
        let mut next =
            self.next_effect_instance_id.max(*allocation_floor).max(largest.checked_add(1).ok_or("Effect instance identity is out of range")?).max(1);
        next.checked_add(missing).ok_or("Effect instance identity space is exhausted")?;
        let mut changed = 0;
        for item in self.items.values_mut() {
            if let ItemKind::Sequence(sequence) = &mut item.kind {
                let mut needs = false;
                visit_sequence(sequence, &mut |effects| {
                    needs |= needs_ids(effects);
                    Ok(())
                })?;
                if !needs {
                    continue;
                }
                let sequence = Arc::make_mut(sequence);
                assign(&mut sequence.master_effects, &mut next, &mut changed)?;
                for track in sequence.video_tracks.iter_mut().chain(&mut sequence.audio_tracks).chain(&mut sequence.submix_tracks) {
                    assign(&mut track.effects, &mut next, &mut changed)?;
                    for clip in &mut track.items {
                        assign(&mut clip.effects, &mut next, &mut changed)?;
                    }
                    for transition in &mut track.transitions {
                        assign(std::slice::from_mut(&mut transition.effect), &mut next, &mut changed)?;
                    }
                }
            }
        }
        for source in self.source_graphics.values_mut() {
            assign(&mut source.layers, &mut next, &mut changed)?;
        }
        self.next_effect_instance_id = next;
        *allocation_floor = next;
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ItemId, SequenceSettings, SourceGraphic, find_effect};

    fn fixture() -> Project {
        let mut project = Project::new("Identity");
        let effect = find_effect("gaussian_blur").unwrap().instance();
        project.source_graphics.insert(ItemId(1), SourceGraphic { layers: vec![effect.clone(), effect], ..Default::default() });
        project
    }

    #[test]
    fn missing_ids_become_distinct_without_changing_effect_content() {
        let mut project = fixture();
        let before = project.source_graphics[&ItemId(1)].layers[0].clone();
        let mut floor = 1;
        assert_eq!(project.assign_effect_instance_ids(&mut floor, false).unwrap(), 2);
        let layers = &project.source_graphics[&ItemId(1)].layers;
        assert_ne!(layers[0].instance_id, 0);
        assert_ne!(layers[0].instance_id, layers[1].instance_id);
        assert_eq!(layers[0].params, before.params);
        assert_eq!(layers[0].masks, before.masks);
        let saved = project.clone();
        assert_eq!(project.assign_effect_instance_ids(&mut floor, false).unwrap(), 0);
        assert_eq!(project, saved);
    }

    #[test]
    fn processing_equality_ignores_owner_identity_but_not_parameters() {
        let mut a = crate::find_effect("gaussian_blur").unwrap().instance();
        a.instance_id = 7;
        let mut b = a.fresh_copy();
        assert_ne!(a, b);
        assert!(a.same_content(&b));
        b.params.get_mut("blurriness").unwrap().value = crate::ParamValue::Float(18.0);
        assert!(!a.same_content(&b));
    }

    #[test]
    fn duplicate_metadata_is_repaired_only_at_explicit_legacy_load_boundary() {
        let mut project = fixture();
        for layer in &mut project.source_graphics.get_mut(&ItemId(1)).unwrap().layers {
            layer.instance_id = 4;
        }
        let before = project.clone();
        let mut floor = 1;
        assert!(project.assign_effect_instance_ids(&mut floor, false).is_err());
        assert_eq!(project, before);
        assert_eq!(floor, 1);
        assert_eq!(project.assign_effect_instance_ids(&mut floor, true).unwrap(), 1);
        assert_eq!(project.source_graphics[&ItemId(1)].layers[0].instance_id, 4);
        assert!(project.source_graphics[&ItemId(1)].layers[1].instance_id > 4);
    }

    #[test]
    fn copied_owner_can_retain_its_internal_ids() {
        let mut project = fixture();
        let mut floor = 1;
        project.assign_effect_instance_ids(&mut floor, false).unwrap();
        let source = project.source_graphics[&ItemId(1)].clone();
        project.source_graphics.insert(ItemId(2), source.clone());
        project.assign_effect_instance_ids(&mut floor, false).unwrap();
        assert_eq!(project.source_graphics[&ItemId(2)], source);
    }

    #[test]
    fn exhausted_or_hostile_ids_fail_before_mutating_anything() {
        for corrupt_id in [0, u64::MAX] {
            let mut project = fixture();
            project.source_graphics.get_mut(&ItemId(1)).unwrap().layers[0].instance_id = corrupt_id;
            let before = project.clone();
            let mut floor = u64::MAX;
            assert!(project.assign_effect_instance_ids(&mut floor, true).is_err());
            assert_eq!(project, before);
        }
    }

    #[test]
    fn assigning_new_ids_does_not_clone_an_unrelated_sequence() {
        let mut project = fixture();
        let id = project.new_sequence("Unrelated", SequenceSettings::default(), 1, 1, None);
        let ItemKind::Sequence(untouched) = &project.item(id).unwrap().kind else { panic!() };
        let untouched = untouched.clone();
        let mut floor = 1;
        project.assign_effect_instance_ids(&mut floor, false).unwrap();
        let ItemKind::Sequence(after) = &project.item(id).unwrap().kind else { panic!() };
        assert!(Arc::ptr_eq(&untouched, after));
    }
}
