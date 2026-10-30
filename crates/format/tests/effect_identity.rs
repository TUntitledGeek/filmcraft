use filmcraft_format::{SCHEMA_VERSION, decode, encode};
use filmcraft_project::{ItemId, Project, SourceGraphic, find_effect};
use serde_json::json;

fn project() -> Project {
    let mut p = Project::new("Legacy duplicate effects");
    let mut a = find_effect("gaussian_blur").unwrap().instance();
    let mut b = a.clone();
    a.param_mut("blurriness").unwrap().value = filmcraft_project::ParamValue::Float(8.0);
    b.param_mut("blurriness").unwrap().value = filmcraft_project::ParamValue::Float(21.0);
    p.source_graphics.insert(ItemId(1), SourceGraphic { layers: vec![a, b], ..Default::default() });
    p
}

#[test]
fn pre_identity_project_upgrades_once_and_preserves_order_values_and_saved_ids() {
    let original = project();
    let mut legacy: serde_json::Value = serde_json::from_slice(&encode(&original, false)).unwrap();
    legacy["schema_version"] = json!(15);
    legacy["project"].as_object_mut().unwrap().remove("next_effect_instance_id");
    let loaded = decode(&serde_json::to_vec(&legacy).unwrap()).unwrap();
    assert!(loaded.migrated());
    let effects = &loaded.project.source_graphics[&ItemId(1)].layers;
    assert_ne!(effects[0].instance_id, 0);
    assert_ne!(effects[0].instance_id, effects[1].instance_id);
    for (old, new) in original.source_graphics[&ItemId(1)].layers.iter().zip(effects) {
        assert_eq!(old.params, new.params);
        assert_eq!(old.masks, new.masks);
        assert_eq!(old.effect, new.effect);
    }
    let reopened = decode(&encode(&loaded.project, false)).unwrap();
    assert_eq!(reopened.schema_version, SCHEMA_VERSION);
    assert!(!reopened.migrated());
    assert_eq!(reopened.project, loaded.project);
}

#[test]
fn malformed_duplicate_metadata_is_repaired_without_swapping_effect_content() {
    let mut p = project();
    for effect in &mut p.source_graphics.get_mut(&ItemId(1)).unwrap().layers {
        effect.instance_id = 9;
    }
    let loaded = decode(&encode(&p, false)).unwrap();
    let effects = &loaded.project.source_graphics[&ItemId(1)].layers;
    assert_eq!(effects[0].instance_id, 9);
    assert_ne!(effects[1].instance_id, 9);
    assert_eq!(effects[0].f64_at("blurriness", filmcraft_time::Tick::ZERO), 8.0);
    assert_eq!(effects[1].f64_at("blurriness", filmcraft_time::Tick::ZERO), 21.0);
}

#[test]
fn hostile_identity_numbers_return_errors_without_panicking() {
    let mut p = project();
    p.source_graphics.get_mut(&ItemId(1)).unwrap().layers[0].instance_id = u64::MAX;
    let result = std::panic::catch_unwind(|| decode(&encode(&p, false)));
    assert!(result.is_ok());
    assert!(result.unwrap().is_err());
}
