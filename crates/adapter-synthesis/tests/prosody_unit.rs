//! Prosody unit tests (015).

use adapter_synthesis::chunk::split_document;
use adapter_synthesis::prosody::{ProsodyPlan, plan_for_segment};

#[test]
fn heading_detected() {
    let segs = split_document("# Title\nbody", "");
    assert_ne!(segs.len(), 0);
    assert_eq!(plan_for_segment(&segs[0]), ProsodyPlan::Heading);
}

#[test]
fn thematic_break_detected() {
    let segs = split_document("---", "");
    assert_ne!(segs.len(), 0);
    assert_eq!(plan_for_segment(&segs[0]), ProsodyPlan::ThematicBreak);
}

#[test]
fn code_detected() {
    let segs = split_document("```\ncode\n```", "");
    assert_ne!(segs.len(), 0);
    assert_eq!(plan_for_segment(&segs[0]), ProsodyPlan::Code);
}

#[test]
fn question_detected() {
    let segs = split_document("Is this a question?", "");
    assert_ne!(segs.len(), 0);
    assert_eq!(plan_for_segment(&segs[0]), ProsodyPlan::Question);
}

#[test]
fn exclamation_detected() {
    let segs = split_document("Wow!", "");
    assert_ne!(segs.len(), 0);
    assert_eq!(plan_for_segment(&segs[0]), ProsodyPlan::Exclamation);
}

#[test]
fn list_detected() {
    let segs = split_document("* item one", "");
    assert_ne!(segs.len(), 0);
    assert_eq!(plan_for_segment(&segs[0]), ProsodyPlan::List);
}

#[test]
fn ordered_list_detected() {
    let segs = split_document("1. item one", "");
    assert_ne!(segs.len(), 0);
    assert_eq!(plan_for_segment(&segs[0]), ProsodyPlan::List);
}

#[test]
fn strong_detected() {
    let segs = split_document("**bold** text", "");
    assert_ne!(segs.len(), 0);
    assert_eq!(plan_for_segment(&segs[0]), ProsodyPlan::Strong);
}

#[test]
fn underscore_strong_detected() {
    let segs = split_document("__bold__ text", "");
    assert_ne!(segs.len(), 0);
    assert_eq!(plan_for_segment(&segs[0]), ProsodyPlan::Strong);
}

#[test]
fn highlight_detected_as_emphasis() {
    let segs = split_document("==highlight== text", "");
    assert_ne!(segs.len(), 0);
    assert_eq!(plan_for_segment(&segs[0]), ProsodyPlan::Emphasis);
}

#[test]
fn quote_detected() {
    let segs = split_document("> quote text", "");
    assert_ne!(segs.len(), 0);
    assert_eq!(plan_for_segment(&segs[0]), ProsodyPlan::Quote);
}

#[test]
fn speed_values_valid() {
    assert!(ProsodyPlan::Code.speed() < 1.0);
    assert!(ProsodyPlan::Heading.speed() < 1.0);
    assert_eq!(ProsodyPlan::Normal.speed(), 1.0);
    assert!(ProsodyPlan::Code.speed() > 0.0);
}

#[test]
fn normal_for_plain_text() {
    let segs = split_document("Just a plain paragraph.", "");
    assert_ne!(segs.len(), 0);
    assert_eq!(plan_for_segment(&segs[0]), ProsodyPlan::Normal);
}
