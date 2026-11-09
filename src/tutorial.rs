//! Tutorials, each taken up from its poster on a building's wall (Henry, 2026-11-08: all of them
//! optional, none of them in the way). A poster teaches one thing, a step at a time in the HUD;
//! `game.rs` decides when each step is met. Most checks look at world state rather than single
//! events, so doing things early still counts: a step already done is skipped. A new player gets
//! one welcome, which says where the posters are; after that nothing asks anything of them.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// One tutorial: what its poster is called, what it teaches, its steps, and anything said once
/// it's done beyond that it's done.
pub struct Topic {
    pub title: &'static str,
    pub about: &'static str,
    pub steps: &'static [&'static str],
    pub after: &'static str,
}

// Each poster's tutorial, by its place in `TOPICS`.
pub const CLOTH: usize = 0;
pub const LOOPS: usize = 1;
pub const HELPERS: usize = 2;
pub const FOOTBRIDGE: usize = 3;
pub const DYEING: usize = 4;
pub const WAGONS: usize = 5;
pub const KITS: usize = 6;

pub const TOPICS: [Topic; 7] = [
    Topic {
        title: "MAKING CLOTH",
        about: "COTTON INTO CLOTH: HARVEST IT, CARD IT, SPIN IT, WEAVE IT AND STORE IT FOR RENOWN.",
        steps: &[
            "STAND ON A RIPE COTTON PLANT, FACE THE BIN, AND PRESS SPACE TO HARVEST.",
            "PUSH A COTTON BOLL INTO THE COTTON BIN.",
            "THE BIN'S CHUTE FEEDS THE WORKBENCH: PUSH A BOLL ONTO ITS PAD, STAND AT THE BENCH, PRESS SPACE.",
            "PUSH THE SLIVERS EAST INTO THE SPINNERY AND SPIN THEM INTO THREAD.",
            "PUSH THE THREAD EAST TO THE LOOM HOUSE NEXT DOOR AND WEAVE FABRIC.",
            "PUSH THE FABRIC INTO THE CLOTH STORE BESIDE THE LOOM HOUSE TO EARN RENOWN.",
        ],
        after: "",
    },
    Topic {
        title: "TEACHING LOOPS",
        about: "SHOW A MONSTER PART OF THE JOB ONCE, AND IT KEEPS ON DOING IT.",
        steps: &["TEACH A MONSTER PART OF THE JOB: R, DO IT, R AGAIN. IT WALKS BACK TO ITS FLAG.", "PRESS P TO LOOP IT. THEN TAB TO THE OTHER MONSTER AND TEACH THE NEXT PART."],
        after: "",
    },
    Topic {
        title: "HELPERS",
        about: "MORE MONSTERS FOR YOUR COMMUNE, WELCOMED FOR RENOWN.",
        steps: &["SAVE UP RENOWN, THEN WELCOME A HELPER AT THE GATHERING HALL, BELOW THE COTTON FIELD."],
        after: "",
    },
    Topic {
        title: "THE FOOTBRIDGE",
        about: "ROPE FOR A FOOTBRIDGE OVER THE RIVER, AND THE ROAD EAST TO TANGLEMERE.",
        steps: &[
            "THE FOOTBRIDGE EAST NEEDS ROPE: PUSH TWO THREAD ONTO THE BENCH BY THE EAST GATE AND TWIST THEM.",
            "PUSH THREE ROPES INTO THE FOOTBRIDGE'S BOARD, OUT THE EAST GATE BY THE RIVER.",
            "THE FOOTBRIDGE IS UP! CROSS IT AND FOLLOW THE ROAD EAST TO TANGLEMERE.",
        ],
        after: "",
    },
    Topic {
        title: "DYEING WITH INDIGO",
        about: "TANGLEMERE'S BLUE: LYE AND QUICKLIME FROM THE KILN, INDIGO, THEN THE DYE HOUSE.",
        steps: &[
            "TANGLEMERE DYES CLOTH BLUE. AT ITS KILN, BURN RAMIE INTO LYE. STICKS AND LOGS ARE FUEL.",
            "LIMESTONE FROM THE THATCH HILLS BURNS INTO QUICKLIME AT THE KILN TOO.",
            "MIX INDIGO AT THE INDIGO HOUSE: INDIGOFERA LEAVES, LYE, TWO QUICKLIME AND FUEL.",
            "DYE FABRIC WITH INDIGO AT THE DYE HOUSE, THEN TAKE IT TO THE CLOTH HALL.",
        ],
        after: "",
    },
    Topic {
        title: "WAGONS",
        about: "LEADING PINEWATER'S WAGON, LOADING AND UNLOADING IT, AND A BRIDGE IT CAN CROSS.",
        steps: &[
            "PINEWATER LENDS A WAGON, BY THE SOUTHEAST GATE. FACE ITS PUMPKIN AND PRESS SPACE TO LEAD IT.",
            "PUSH GOODS INTO THE WAGON. PARKED, SPACE AT ITS BACK OPENS THE TAILGATE AND IT UNLOADS.",
            "WAGONS DON'T FIT ON THE FOOTBRIDGE. PUSH 12 LOGS INTO THE BRIDGE BOARD, SOUTH ON THE FAR BANK.",
        ],
        after: "",
    },
    Topic {
        title: "KITS AND THE WILDS",
        about: "A WORKSHOP IN A KIT, SET UP OUT IN THE WILDS BY ITS MATERIALS.",
        steps: &[
            "AT THE BUILDERS' BENCH BY THE SOUTHEAST GATE, MAKE A KILN KIT: 3 STONE, QUICKLIME, ROPE, A LOG.",
            "TAKE THE KIT OUT OF TOWN, OFF THE ROAD. FACE IT AND PRESS SPACE TO SET UP A WORKSHOP BY ITS MATERIALS.",
        ],
        after: "WORKSHOPS OUT THERE WEAR OUT: PUSH A NEW KIT ONTO ITS WORK SPOT TO RENEW IT.",
    },
];

/// The HUD's first line while no tutorial is taken up.
pub const IDLE: &str = "F1 SHOWS THE CONTROLS. POSTERS ON BUILDING WALLS EACH TEACH ONE THING.";

/// What a new player is told once, on arriving, before anything else; any key closes it.
pub const WELCOME: [&str; 6] = [
    "ARROWS OR WASD WALK. WALK INTO THINGS TO PUSH THEM.",
    "HOLD E TO EXAMINE WHATEVER YOU FACE.",
    "POSTERS ON BUILDING WALLS EACH TEACH ONE THING:",
    "FACE ONE AND PRESS SPACE TO TAKE IT UP. ALL OPTIONAL.",
    "THE FIRST IS ON THE GATHERING HALL, BELOW THE COTTON.",
    "F1 SHOWS THE CONTROLS.",
];

pub const HELP: [&str; 12] = [
    "ARROWS / WASD   WALK, PUSH",
    "SHIFT + ARROW   PULL (SLOW)",
    "SPACE           USE: HARVEST, CRAFT",
    "E (HOLD)        EXAMINE (TAB: MORE)",
    "R               RECORD / FINISH (WALKS HOME)",
    "BACKSPACE       CANCEL RECORDING",
    "P               LOOP / STOP (ON A FLAG: COPY)",
    "TAB             SWITCH MONSTER (HOLD: ROSTER)",
    "C               CHARACTER SHEET (HANDS, LOOP)",
    "F1              THIS HELP",
    "F2              CRT SCANLINES",
    "ESC             CLOSE, OR LEAVE",
];

/// A player's tutorials: the one taken up, if any, and the step it's on; and those seen through.
#[derive(Default, Serialize, Deserialize)]
pub struct Tutorial {
    pub on: Option<(usize, usize)>,
    pub done: BTreeSet<usize>,
}

/// Splits text into lines of at most `max` characters on word boundaries.
pub fn wrap(text: &str, max: usize) -> Vec<String> {
    // A number goes with what it counts: "+1 YIELD!" never breaks after the "+1".
    let number = |w: &str| {
        let digits = w.trim_start_matches(['+', '-']).trim_end_matches('%');
        !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
    };
    let mut lines = vec![String::new()];
    for word in text.split_whitespace() {
        let cur = lines.last_mut().expect("at least one line");
        if !cur.is_empty() && cur.len() + 1 + word.len() > max {
            let next = match cur.rsplit_once(' ') {
                Some((head, n)) if number(n) && n.len() + 1 + word.len() <= max => {
                    let next = format!("{n} {word}");
                    *cur = head.to_string();
                    next
                }
                _ => word.to_string(),
            };
            lines.push(next);
        } else {
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(word);
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_step_fits_two_hud_lines_and_the_welcome_fits_its_box() {
        for s in TOPICS.iter().flat_map(|t| t.steps.iter()).chain([&IDLE]) {
            assert!(wrap(s, 64).len() <= 2, "too long for the HUD: {s}");
        }
        for t in &TOPICS {
            assert!(!t.steps.is_empty() && t.title.len() <= 30, "{}", t.title);
        }
        assert!(WELCOME.iter().all(|l| l.len() <= 60), "too wide for the welcome");
    }
}
