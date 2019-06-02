use palette::Srgb;
use regex::Regex;

pub fn parse_color(color: &str) -> Option<Srgb<u8>> {
    // RRGGBB
    let re_rrggbb = Regex::new(r"^#?([[:xdigit:]]{2})([[:xdigit:]]{2})([[:xdigit:]]{2})$").unwrap();

    if let Some(caps) = re_rrggbb.captures(color) {
        let rr = caps.get(1).unwrap().as_str();
        let gg = caps.get(2).unwrap().as_str();
        let bb = caps.get(3).unwrap().as_str();

        let rr = u8::from_str_radix(rr, 16).unwrap();
        let gg = u8::from_str_radix(gg, 16).unwrap();
        let bb = u8::from_str_radix(bb, 16).unwrap();

        return Some(Srgb::from_components((rr, gg, bb)));
    }

    None
}
