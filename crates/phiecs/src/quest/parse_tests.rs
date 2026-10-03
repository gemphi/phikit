//! The shapes real models answer in.
use super::parse::*;

mod shapes {
    use super::*;

    /// The sixteen kinds a quest offers first.
    const KINDS: &str = "is-a part-of has does done-to like unlike near before after causes used-for made-of belongs-to more-than less-than";

    fn offered() -> Vec<String> {
        KINDS.split(' ').map(String::from).collect()
    }

    fn one(text: &str) -> Vec<(String, String, f64)> {
        parse_relations(text, &offered()).into_iter().map(|c| (c.kind, c.other, c.weight)).collect()
    }

    fn c(k: &str, o: &str, w: f64) -> (String, String, f64) {
        (k.into(), o.into(), w)
    }

    #[test]
    fn a_markdown_table_reads_like_plain_lines() {
        let t = "| kind | other word | weight |\n|---|---|---|\n| is-a | animal | 0.9 |\n| has | fur | 0.8 |";
        assert_eq!(one(t), vec![c("is-a", "animal", 0.9), c("has", "fur", 0.8)]);
    }

    #[test]
    fn numbering_does_not_invent_kinds() {
        assert_eq!(one("1. is-a | animal | 0.9\n2) has | fur | 0.8"), vec![c("is-a", "animal", 0.9), c("has", "fur", 0.8)]);
    }

    #[test]
    fn one_relation_keeps_one_name() {
        for k in ["used-for", "used_for", "Used For", "**used for**"] {
            assert_eq!(one(&format!("{k} | hunting | 0.5")), vec![c("used-for", "hunting", 0.5)], "{k}");
        }
    }

    #[test]
    fn a_weight_is_the_number_it_starts_with() {
        assert_eq!(one("is-a | animal | 0.9 (strong)"), vec![c("is-a", "animal", 0.9)]);
        assert_eq!(one("is-a | animal | 1.7"), vec![c("is-a", "animal", 1.0)]);
    }

    #[test]
    fn headers_and_separators_are_not_relations() {
        assert!(one("kind | word | weight").is_empty());
        assert!(one("|---|---|---|").is_empty());
        assert!(one("|:---|:---:|---:|").is_empty());
    }

    #[test]
    fn phi4_drops_the_pipes() {
        // Verbatim from phi4_cat/steps/00{1,2}-*.md, trailing spaces included.
        let t = "is-a mammal 0.9  \nbelongs-to Felidae 0.8  \nlikes fish 0.6  \ndoes give birth 0.7  \n\
                 has mammary glands 0.9  \nused for hunting 0.5\ncauses allergy 0.5";
        assert_eq!(
            one(t),
            vec![
                c("is-a", "mammal", 0.9),
                c("belongs-to", "felidae", 0.8),
                c("like", "fish", 0.6),
                c("does", "give-birth", 0.7),
                c("has", "mammary-glands", 0.9),
                c("used-for", "hunting", 0.5),
                c("causes", "allergy", 0.5),
            ]
        );
        assert_eq!(one("is-a\tmammal\t0.9"), vec![c("is-a", "mammal", 0.9)]);
        assert!(one("A cat is a small domesticated mammal.").is_empty());
        assert!(one("Here are the relations for cat 8").is_empty());
    }

    #[test]
    fn phi4s_other_shapes() {
        // Verbatim from phi4_cat/steps/00{5,7,8}-*.md.
        let fish = "kind: is-a, other word: aquatic, weight: 0.9  \nkind: has, other word: scales, weight: 0.8";
        assert_eq!(one(fish), vec![c("is-a", "aquatic", 0.9), c("has", "scales", 0.8)]);
        let purr = "does  \nsound  \n1.0  \n\nis-a  \nnoise  \n0.9  \n\nlike  \ncomfort  \n0.6";
        assert_eq!(one(purr), vec![c("does", "sound", 1.0), c("is-a", "noise", 0.9), c("like", "comfort", 0.6)]);
        let dog = "has-bark | bark | 0.8  \nhas-tail | tail | 0.8  \nbelongs-to | owner | 0.5";
        assert_eq!(one(dog), vec![c("has", "bark", 0.8), c("has", "tail", 0.8), c("belongs-to", "owner", 0.5)]);
    }

    #[test]
    fn a_kind_not_offered_is_dropped() {
        let t = "protects | skin | 0.8\nfound-on | mammal | 0.9\nbelongsto | family | 0.7\nmade-of | hair | 0.9";
        assert_eq!(one(t), vec![c("belongs-to", "family", 0.7), c("made-of", "hair", 0.9)]);
        assert_eq!(one("opposite | stress | 0.9\nhas-a | license | 0.5"), vec![c("unlike", "stress", 0.9), c("has", "license", 0.5)]);
    }

    #[test]
    fn a_compound_other_is_one_word() {
        assert_eq!(one("near | pet store | 0.6"), vec![c("near", "pet-store", 0.6)]);
        assert!(one("near | a long phrase here | 0.6").is_empty());
    }

    #[test]
    fn a_kind_bent_by_one_s_is_the_offered_kind() {
        let offered: Vec<String> = ["like", "causes", "has"].iter().map(|s| s.to_string()).collect();
        assert_eq!(offered_kind("likes", &offered), "like");
        assert_eq!(offered_kind("cause", &offered), "causes");
        assert_eq!(offered_kind("has", &offered), "has");
        assert_eq!(offered_kind("eats", &offered), "eats");
    }

    #[test]
    fn crlf_and_bold_survive() {
        assert_eq!(one("**is-a** | **animal** | 0.9\r\nhas | fur | 0.8\r\n"), vec![c("is-a", "animal", 0.9), c("has", "fur", 0.8)]);
    }
}
