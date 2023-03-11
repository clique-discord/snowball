use snowball::System;

fn test1() {
    let mut system = System::new();
    system.add_node(0, [181, 137, 0]);
    system.add_node(1, [203, 75, 22]);
    system.add_node(2, [220, 50, 47]);
    system.add_node(3, [211, 54, 130]);
    system.add_node(4, [108, 113, 196]);
    system.many_steps(150);
    system.set_weight(0, 1, 50.);
    system.many_steps(150);
    system.set_weight(1, 2, 200.);
    system.many_steps(150);
    system.set_weight(1, 3, 70.);
    system.many_steps(150);
    system.set_weight(2, 4, 5000.);
    system.many_steps(150);
    system.set_weight(0, 3, 200.);
    system.many_steps(150);
    system.add_node(5, [38, 139, 210]);
    system.many_steps(150);
    system.add_node(6, [42, 161, 152]);
    system.many_steps(150);
    system.set_weight(5, 6, 60.);
    system.many_steps(150);
    system.add_node(7, [133, 153, 0]);
    system.many_steps(150);
    system.set_weight(6, 7, 200.);
    system.many_steps(150);
    system.set_weight(5, 7, 50.);
    system.many_steps(150);
    system.set_weight(1, 7, 5000.);
    system.many_steps(400);
    #[cfg(feature = "lottie")]
    system.render_lottie();
}

fn main() {
    let n = std::env::args()
        .nth(1)
        .map(|s| s.parse().unwrap())
        .unwrap_or(1);
    for _ in 0..n {
        test1();
    }
}
