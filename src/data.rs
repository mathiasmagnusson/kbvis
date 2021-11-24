use std::collections::HashMap;

const DATA: &str = include_str!("../ngrams1.tsv");

pub fn get_1_gram_distrib() -> HashMap<char, f32> {
    let mut list = vec![];
    let mut max: usize = 0;

    for line in DATA.split('\n').skip(1).take_while(|line| !line.starts_with("2-gram")) {
        let mut cells = line.split('\t');

        let ch = cells.next().unwrap().chars().next().unwrap();
        let occ = cells.next().unwrap().parse::<usize>().unwrap();
        max = max.max(occ);

        list.push((ch, occ));
    }

    list.into_iter().map(|(ch, occ)| (ch, occ as f32 / max as f32)).collect()
}

pub fn get_2_gram_distrib() -> HashMap<(char, char), f32> {
    let mut list = vec![];
    let mut max: usize = 0;

    for line in DATA.split('\n').skip_while(|line| !line.starts_with("2-gram")).skip(1).take_while(|line| !line.starts_with("3-gram")) {
        let mut cells = line.split('\t');

        let mut cs = cells.next().unwrap().chars();
        let fst = cs.next().unwrap();
        let snd = cs.next().unwrap();
        let occ = cells.next().unwrap().parse::<usize>().unwrap();
        max = max.max(occ);

        list.push(((fst, snd), occ));
    }

    list.into_iter().map(|(cs, occ)| (cs, occ as f32 / max as f32)).collect()
}
