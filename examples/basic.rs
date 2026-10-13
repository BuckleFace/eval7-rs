use eval7_rs::{Card, HandCategory, evaluate_7};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let straight_flush = ["As", "Ks", "Qs", "Js", "Ts", "2d", "3c"]
        .map(|text| text.parse::<Card>().expect("valid card literal"));
    let pair = ["Ac", "Ad", "Kh", "9s", "7c", "4d", "2h"]
        .map(|text| text.parse::<Card>().expect("valid card literal"));

    let straight_flush_rank = evaluate_7(straight_flush)?;
    let pair_rank = evaluate_7(pair)?;

    assert_eq!(straight_flush_rank.category(), HandCategory::StraightFlush);
    assert!(straight_flush_rank > pair_rank);
    println!("packed rank: {}", straight_flush_rank.raw());
    Ok(())
}
