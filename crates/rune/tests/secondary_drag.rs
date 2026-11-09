//! `El::pointer_buttons([PointerButton::Secondary])`: an element that opts in hears a
//! secondary-button press as `on_press`/`on_drag` (a ruler's right-drag on a map), while elements
//! that don't keep opening `on_secondary_press`, their context menu.
use rune::{Cx, Drag, El, Env, Phase, PointerButton, PointerPress, Runtime, Tx, el};

#[derive(Default)]
struct S {
    /// The map's drag: each phase, the button, and the drag's offset.
    drags: Vec<(Phase, PointerButton, f32, f32)>,
    map_menus: usize,
    row_menus: Vec<PointerPress>,
}

fn view(_: &S, _: &mut Cx<S>) -> El {
    let map = el().key("map").pin(Some(0.0), Some(0.0), None, None).size(300.0, 300.0)
        .pointer_buttons([PointerButton::Secondary])
        .on_drag(|s: &mut S, d: &mut Drag, tx: &mut Tx| s.drags.push((d.phase, tx.button(), d.dx, d.dy)))
        .on_secondary_press(|s: &mut S, _, _| s.map_menus += 1);
    let row = el().key("row").pin(Some(0.0), Some(320.0), None, None).size(300.0, 40.0)
        .on_secondary_press(|s: &mut S, p, _| s.row_menus.push(p));
    el().key("root").size(400.0, 400.0).child(map).child(row)
}

fn app() -> Runtime<S> {
    let mut rt = Runtime::new(S::default(), view, (400.0, 400.0), Env::new(true, 1.0));
    settle(&mut rt);
    rt
}

fn settle(rt: &mut Runtime<S>) {
    for _ in 0..600 { if !rt.frame(1.0 / 120.0) && !rt.has_pending_input() { return; } }
    panic!("never settled");
}

#[test]
fn an_element_that_opts_in_is_dragged_by_the_secondary_button() {
    let mut rt = app();
    rt.secondary_down(100.0, 100.0);
    settle(&mut rt);
    for i in 1..=5 {
        rt.pointer_move(100.0 + 10.0 * i as f32, 100.0 + 4.0 * i as f32);
        rt.frame(1.0 / 120.0);
    }
    rt.secondary_up(150.0, 120.0);
    settle(&mut rt);
    let s = &rt.state;
    assert_eq!(s.map_menus, 0, "a right-drag on an element that takes the button opens no menu");
    assert!(s.drags.iter().all(|d| d.1 == PointerButton::Secondary), "{:?}", s.drags);
    assert_eq!(s.drags.first().map(|d| d.0), Some(Phase::Down));
    assert_eq!(s.drags.last().map(|d| d.0), Some(Phase::Up));
    let (_, _, dx, dy) = *s.drags.last().unwrap();
    assert!((dx - 50.0).abs() < 0.5 && (dy - 20.0).abs() < 0.5, "drag offset {dx}, {dy}");
}

#[test]
fn elements_that_do_not_opt_in_keep_their_context_menu() {
    let mut rt = app();
    rt.secondary_down(50.0, 340.0);
    rt.secondary_up(50.0, 340.0);
    settle(&mut rt);
    let s = &rt.state;
    assert_eq!(s.row_menus.len(), 1);
    assert!(s.drags.is_empty());
}

#[test]
fn other_buttons_still_need_their_own_opt_in() {
    let mut rt = app();
    rt.button_down(100.0, 100.0, PointerButton::Middle);
    rt.pointer_move(130.0, 100.0);
    rt.button_up(130.0, 100.0, PointerButton::Middle);
    settle(&mut rt);
    assert!(rt.state.drags.is_empty(), "the map took only the secondary button");
}
