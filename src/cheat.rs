//! Cheats, for testing. In the window, ` opens a line; a good's name and Enter set that good
//! down beside the monster you control, on open ground ("3 OAK PLANKS": a count first, and a
//! variety before the name, are both optional). Tab fills in the first name it suggests. Only a
//! world played alone takes them: a server elsewhere says no.

use crate::data::{Data, GoodType};
use crate::econ::ItemData;
use crate::game::{Game, Heard, Sfx};
use crate::world::{Item, DIRS};

/// The most goods one cheat sets down.
pub const MOST: u32 = 20;
/// The longest line the window takes.
pub const LONGEST: usize = 40;
/// How far from the monster, in steps, a good may be set down.
const REACH: usize = 6;
/// Names the window suggests at once.
const HINTS: usize = 4;

/// What a line asks for: how many of which good, of which variety ("oak").
#[derive(Debug, PartialEq)]
pub struct Ask {
    pub count: u32,
    pub good: String,
    pub variety: Option<String>,
}

/// Words as they're compared: capitals, with anything but letters and digits a space between
/// them ("Green Vitriol (found)" is "GREEN VITRIOL FOUND", and so is green_vitriol_found).
fn words(text: &str) -> String {
    let spaced: String = text.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() } else { ' ' }).collect();
    spaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The count at the front, if any, and the rest.
fn count_and_rest(text: &str) -> (Option<u32>, String) {
    let w = words(text);
    let (n, rest) = w.split_once(' ').unwrap_or((w.as_str(), ""));
    if !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()) {
        return (Some(n.parse::<u32>().unwrap_or(MOST).clamp(1, MOST)), rest.to_string());
    }
    (None, w.clone())
}

/// Every good a cheat can make: all but trades (each is named for the good it gives).
fn goods(data: &Data) -> impl Iterator<Item = &GoodType> {
    data.goods.iter().filter(|g| g.makes.is_none())
}

/// The varieties `g` can come in: those its recipe carries over from what it's made of.
fn carries(g: &GoodType, variety: &str) -> bool {
    g.input_goods_tags_carryover.iter().flatten().any(|t| t == variety)
}

/// The variety named by `w` (in words), if there's one.
fn variety_named(data: &Data, w: &str) -> Option<String> {
    data.varieties.iter().map(|v| &v.tag).find(|t| words(t) == w).cloned()
}

/// The goods `rest` names exactly, by name or id, with a variety in front or not; a trailing
/// S is forgiven ("NAILS").
fn exact(data: &Data, rest: &str) -> Option<(String, Option<String>)> {
    let tries = std::iter::once(rest.to_string()).chain(rest.strip_suffix('S').map(str::to_string));
    for t in tries {
        for g in goods(data) {
            for name in [words(&g.name), words(&g.id)] {
                if t == name {
                    return Some((g.id.clone(), None));
                }
                let Some(front) = t.strip_suffix(&name).and_then(|f| f.strip_suffix(' ')) else { continue };
                if let Some(v) = variety_named(data, front) {
                    return Some((g.id.clone(), Some(v)));
                }
            }
        }
    }
    None
}

/// What a line asks for, or why it can't be done. A name only part typed does, if only one good's
/// name has it.
pub fn read(data: &Data, text: &str) -> Result<Ask, String> {
    let (count, rest) = count_and_rest(text);
    let count = count.unwrap_or(1);
    if rest.is_empty() {
        return Err("NAME A GOOD.".into());
    }
    if let Some((good, variety)) = exact(data, &rest) {
        return Ok(Ask { count, good, variety });
    }
    let some: Vec<&GoodType> = goods(data).filter(|g| words(&g.name).contains(&rest)).collect();
    match some.as_slice() {
        [] => Err(format!("NO GOOD IS CALLED {rest}.")),
        [g] => Ok(Ask { count, good: g.id.clone(), variety: None }),
        many => {
            let names: Vec<String> = many.iter().take(3).map(|g| g.name.to_uppercase()).collect();
            let more = if many.len() > 3 { format!(" AND {} MORE", many.len() - 3) } else { String::new() };
            Err(format!("WHICH? {}{more}.", names.join(", ")))
        }
    }
}

/// One good as a line asks for it: plainly made, of standard quality, from its variety's own
/// good where it names one (oak planks from an oak log).
pub fn make(data: &Data, ask: &Ask) -> Result<ItemData, String> {
    let g = data.good(&ask.good);
    let Some(v) = &ask.variety else { return Ok(crate::econ::plain(data, &g.id, &g.id)) };
    let from = goods(data).filter(|s| s.variety.as_ref().is_some_and(|sv| &sv.name == v)).max_by_key(|s| s.is_raw());
    match from {
        Some(from) if carries(g, v) => Ok(crate::econ::plain(data, &g.id, &from.id)),
        _ => Err(format!("{} DOESN'T COME IN {}.", g.name.to_uppercase(), v.to_uppercase())),
    }
}

/// What the window suggests for a line half typed: the names it could become, the closest
/// first, each with the count and variety typed so far. Tab fills in the first.
pub fn hints(data: &Data, text: &str) -> Vec<String> {
    let (count, rest) = count_and_rest(text);
    if rest.is_empty() {
        return Vec::new();
    }
    // The rest as typed, and after each variety it could start with.
    let mut splits = vec![(None, rest.clone())];
    for v in &data.varieties {
        if let Some(after) = rest.strip_prefix(&words(&v.tag)).and_then(|a| a.strip_prefix(' ')) {
            splits.push((Some(v.tag.clone()), after.to_string()));
        }
    }
    let mut found: Vec<(u8, usize, String)> = Vec::new();
    for (variety, part) in &splits {
        for g in goods(data).filter(|g| variety.as_ref().is_none_or(|v| carries(g, v))) {
            let name = words(&g.name);
            let rank = if name == *part { 0 } else if name.starts_with(part.as_str()) { 1 } else if name.split(' ').any(|w| w.starts_with(part.as_str())) { 2 } else if name.contains(part.as_str()) { 3 } else { continue };
            let shown = [count.map(|n| n.to_string()), variety.as_ref().map(|v| v.to_uppercase()), Some(g.name.to_uppercase())].into_iter().flatten().collect::<Vec<_>>().join(" ");
            found.push((rank, shown.len(), shown));
        }
    }
    found.sort();
    let mut out: Vec<String> = Vec::new();
    for (_, _, s) in found {
        if !out.contains(&s) && out.len() < HINTS {
            out.push(s);
        }
    }
    out
}

impl Game {
    /// Commune `k`'s player typed a cheat: what it asks for is set down beside their monster,
    /// on open ground, the tile it faces first, and its bubble says what came or why nothing did.
    /// Only where `allowed` (a world played alone).
    pub fn cheat(&mut self, k: usize, text: &str, allowed: bool) {
        let c = self.communes[k].controlled;
        let made = if allowed { read(&self.data, text).and_then(|ask| make(&self.data, &ask).map(|item| (ask.count, item))) } else { Err("CHEATS ONLY WORK IN A WORLD PLAYED ALONE.".into()) };
        let (said, warn, sfx) = match made {
            Err(why) => (why, true, Sfx::Deny),
            Ok((count, item)) => {
                let spots = self.open_ground_near(c, count as usize);
                let name = crate::inspect::display_name(self, &item);
                for &(x, y) in &spots {
                    self.world.items.push(Item { hop: 4, ..Item::new(x, y, item.clone()) });
                }
                match spots.len() {
                    0 => ("NO ROOM HERE TO SET IT DOWN.".into(), true, Sfx::Deny),
                    n if n < count as usize => (format!("ROOM FOR ONLY {}.", crate::recipes::several(n, &name)), true, Sfx::Plop),
                    n => (format!("CHEAT: {}.", crate::recipes::several(n, &name)), false, Sfx::Plop),
                }
            }
        };
        self.say(c, &said, warn);
        self.sounds.push((Heard::By(k), sfx));
    }

    /// Up to `n` tiles near monster `c` where a good could lie and nothing would take it: the
    /// nearest first by the way it walks, the tile it faces before the rest, on its own screen.
    fn open_ground_near(&self, c: usize, n: usize) -> Vec<(i32, i32)> {
        let m = &self.monsters[c];
        let screen = crate::render::screen_of(m.x, m.y);
        let mut dirs = DIRS;
        dirs.sort_by_key(|&d| d != m.facing);
        let mut seen = vec![(m.x, m.y)];
        let mut ring = vec![(m.x, m.y)];
        let mut out = Vec::new();
        for _ in 0..REACH {
            let mut next = Vec::new();
            for &(x, y) in &ring {
                for d in dirs {
                    let (dx, dy) = d.delta();
                    let t = (x + dx, y + dy);
                    if seen.contains(&t) || !self.world.walkable(t.0, t.1) || crate::render::screen_of(t.0, t.1) != screen {
                        continue;
                    }
                    seen.push(t);
                    next.push(t);
                    if out.len() < n && self.open_for_a_good(t) {
                        out.push(t);
                    }
                }
            }
            ring = next;
        }
        out
    }

    /// Open ground for a cheat's good: floor with nothing on it, and nothing there that takes
    /// goods (a workshop's pads, pegs and outs, a vine).
    fn open_for_a_good(&self, (x, y): (i32, i32)) -> bool {
        let w = &self.world;
        let taken = w.shops.iter().any(|s| s.pads.contains(&(x, y)) || s.outs.contains(&(x, y)) || s.pegs.contains(&(x, y)));
        w.item_floor(x, y) && w.item_at(x, y).is_none() && self.monster_at(x, y).is_none() && !self.applicant_at(x, y) && !w.cart_at(x, y) && w.plant_at(x, y).is_none() && w.prop_at(x, y).is_none() && w.vine_end(x, y).is_none() && w.vine_over(x, y).is_none() && !taken
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(count: u32, good: &str, variety: Option<&str>) -> Ask {
        Ask { count, good: good.into(), variety: variety.map(str::to_string) }
    }

    #[test]
    fn a_line_names_a_good_with_a_count_and_a_variety_or_not() {
        let d = Data::load();
        assert_eq!(read(&d, "copper ore"), Ok(ask(1, "copper_ore", None)));
        assert_eq!(read(&d, "3 Oak Planks"), Ok(ask(3, "planks", Some("oak"))));
        assert_eq!(read(&d, "2 nails"), Ok(ask(2, "nail", None)), "a trailing S is forgiven");
        assert_eq!(read(&d, "green vitriol (found)"), Ok(ask(1, "green_vitriol_found", None)));
        assert_eq!(read(&d, "green_vitriol_found"), Ok(ask(1, "green_vitriol_found", None)), "ids work too");
        assert_eq!(read(&d, "99 pick").map(|a| a.count), Ok(MOST));
        assert_eq!(read(&d, "temple b"), Ok(ask(1, "temple_bell", None)), "part of one good's name is enough");
        assert!(read(&d, "copper").unwrap_err().starts_with("WHICH? "), "part of several asks which");
        assert_eq!(read(&d, "moonbeams"), Err("NO GOOD IS CALLED MOONBEAMS.".into()));
        assert_eq!(read(&d, "4"), Err("NAME A GOOD.".into()));
    }

    #[test]
    fn a_variety_comes_from_its_own_good_and_only_where_the_recipe_carries_it() {
        let g = Game::new();
        let planks = make(&g.data, &ask(1, "planks", Some("oak"))).unwrap();
        assert_eq!(crate::inspect::display_name(&g, &planks), "OAK PLANKS");
        let knife = make(&g.data, &ask(1, "knife", Some("bronze"))).unwrap();
        assert_eq!(crate::inspect::display_name(&g, &knife), "BRONZE KNIFE");
        assert_eq!(make(&g.data, &ask(1, "copper_ore", Some("oak"))).unwrap_err(), "COPPER ORE DOESN'T COME IN OAK.");
        let ore = make(&g.data, &ask(1, "copper_ore", None)).unwrap();
        assert_eq!((ore.good.as_str(), ore.quality), ("copper_ore", 1.0));
    }

    fn bubble(g: &Game, c: usize) -> (String, bool) {
        let b = g.monsters[c].bubble.as_ref().expect("a bubble");
        (b.text.clone(), b.warn)
    }

    #[test]
    fn a_cheat_sets_goods_down_by_the_monster_the_tile_it_faces_first() {
        let mut g = Game::new();
        let c = g.communes[0].controlled;
        let m = &g.monsters[c];
        let (dx, dy) = m.facing.delta();
        let (at, ahead) = ((m.x, m.y), (m.x + dx, m.y + dy));
        let ahead_open = g.open_for_a_good(ahead);
        let before = g.world.items.len();
        g.cheat(0, "3 oak planks", true);
        let new: Vec<(i32, i32)> = g.world.items[before..].iter().map(|it| (it.x, it.y)).collect();
        assert_eq!(new.len(), 3);
        if ahead_open {
            assert_eq!(new[0], ahead, "the tile it faces first");
        }
        for (k, &t) in new.iter().enumerate() {
            assert!((t.0 - at.0).abs() + (t.1 - at.1).abs() <= REACH as i32 && crate::render::screen_of(t.0, t.1) == crate::render::screen_of(at.0, at.1), "{t:?} is near and on its screen");
            assert!(!new[..k].contains(&t), "one to a tile");
        }
        assert!(g.world.items[before..].iter().all(|it| crate::inspect::display_name(&g, &it.data) == "OAK PLANKS"));
        assert_eq!(bubble(&g, c), ("CHEAT: 3 OAK PLANKS.".into(), false));
        assert!(g.sounds.iter().any(|&(h, s)| matches!(h, Heard::By(0)) && s == Sfx::Plop));

        g.cheat(0, "moonbeams", true);
        assert_eq!(bubble(&g, c), ("NO GOOD IS CALLED MOONBEAMS.".into(), true));
        g.cheat(0, "copper ore", false);
        assert_eq!(bubble(&g, c), ("CHEATS ONLY WORK IN A WORLD PLAYED ALONE.".into(), true));
        assert_eq!(g.world.items.len(), before + 3, "neither set anything down");
    }

    #[test]
    fn hints_put_the_closest_names_first_and_keep_what_was_typed() {
        let d = Data::load();
        let h = hints(&d, "3 oak pla");
        assert_eq!(h.first().map(String::as_str), Some("3 OAK PLANKS"));
        let copper = hints(&d, "copper");
        assert!(copper.len() >= 2 && copper.iter().all(|s| s.contains("COPPER")), "{copper:?}");
        assert!(hints(&d, "").is_empty() && hints(&d, "moonbeams").is_empty());
        for h in hints(&d, "temple") {
            assert!(read(&d, &h).is_ok(), "{h} reads back as it's shown");
        }
    }
}
