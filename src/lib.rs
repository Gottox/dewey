use std::cmp::Ordering::*;
use std::cmp::Ordering;
use std::cmp::min;

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
enum Component {
  Alpha,
  Beta,
  Pre,
  Rc,
  PatchLevel,
  DashOrDot,
  End,
  Num(u64),
  Char(char),
}
use Component::*;

impl Component {
  fn real_cmp(&self, other: &Component) -> Option<Ordering> {
    match min((self, other), (other, self)) {
      (&End, &Num(0)) => Some(Equal),
      (&PatchLevel, &End) => Some(Equal),
      (&DashOrDot, &End) => Some(Equal),
      (&Num(_), &Char(_)) => None,
      (&PatchLevel, &DashOrDot) => None,
      (&PatchLevel, &Num(_)) => None,
      (&DashOrDot, &Num(_)) => None,
      (&DashOrDot, &Char(_)) => None,
      _ => self.partial_cmp(&other),
    }
  }

  fn eat_str<'a>(s: &'a str) -> (Component, &'a str) {
    if s.is_empty() {
      return (End, s);
    }
    // Try Parsing Numbers
    match s.chars()
            .map(|c| c.to_digit(10))
            .take_while(Option::is_some)
            .map(Option::unwrap)
            .fold((0u64, 0), |(n, l), i| (n * 10 + i as u64, l + 1)) {
      (_, 0) => (),
      (n, l) => return (Num(n), &s[l..]),
    }
    // Try Parsing patterns
    for &(p, b) in [(DashOrDot, "."),
                    (DashOrDot, "-"),
                    (Alpha, "alpha"),
                    (Beta, "beta"),
                    (Pre, "pre"),
                    (Rc, "rc"),
                    (PatchLevel, "pl")]
            .iter() {
      if s.starts_with(b) {
        return (p, &s[b.len()..]);
      }
    }
    // Fallback to plain char
    (Char(s.chars().flat_map(char::to_lowercase).nth(0).unwrap()), &s[1..])
  }
}

#[derive(PartialEq, Debug)]
pub struct Version<'a> {
  s: &'a str,
}

impl<'a> Version<'a> {
  fn as_str(&self) -> &'a str {
    self.s
  }
}

impl<'a> PartialOrd for Version<'a> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    match (Component::eat_str(self.as_str()),
           Component::eat_str(other.as_str())) {
      ((End, _), (End, _)) => Some(Equal),
      ((s_component, s_remain), (o_component, o_remain)) => {
        match s_component.real_cmp(&o_component) {
          Some(Equal) => s_remain.version().partial_cmp(&o_remain.version()),
          result => result,
        }
      }
    }
  }
}

pub trait VersionCmp {
  fn version<'a>(&'a self) -> Version<'a>;
  fn ver_cmp(&self, other: &Self) -> Option<Ordering> {
    self.version().partial_cmp(&other.version())
  }
}

impl VersionCmp for str {
  fn version(&self) -> Version {
    Version { s: self }
  }
}

// COMPARE VERSION
#[test]
fn compare_version_1_2_to_1_2() {
  assert_eq!("1.2".ver_cmp("1.2"), Some(Equal));
}
#[test]
fn compare_version_upper_a_to_a() {
  assert_eq!("A".ver_cmp("a"), Some(Equal));
}
#[test]
fn compare_version_a_to_b() {
  assert_eq!("a".ver_cmp("b"), Some(Less));
}
#[test]
fn compare_version_aa_to_b() {
  assert_eq!("aa".ver_cmp("b"), Some(Less));
}
#[test]
fn compare_version_1_to_1() {
  assert_eq!("1".ver_cmp("1"), Some(Equal));
}
#[test]
fn compare_version_1_to_1_0() {
  assert_eq!("1".ver_cmp("1.0"), Some(Equal));
}
#[test]
fn compare_version_1_to_1pl0() {
  assert_eq!("1".ver_cmp("1pl0"), Some(Equal));
}
#[test]
fn compare_version_1_to_0() {
  assert_eq!("1".ver_cmp("0"), Some(Greater));
}
#[test]
fn compare_version_1_to_0_0_1() {
  assert_eq!("1".ver_cmp("0.0.1"), Some(Greater));
}
#[test]
fn compare_version_1_to_1pre1() {
  assert_eq!("1".ver_cmp("1pre1"), Some(Greater));
}
#[test]
fn compare_version_1_to_1rc1() {
  assert_eq!("1".ver_cmp("1rc1"), Some(Greater));
}
#[test]
fn compare_version_1_to_1alpha() {
  assert_eq!("1".ver_cmp("1alpha"), Some(Greater));
}
#[test]
fn compare_version_1_to_1alpha1() {
  assert_eq!("1".ver_cmp("1alpha1"), Some(Greater));
}
#[test]
fn compare_version_1_to_1beta1() {
  assert_eq!("1".ver_cmp("1beta1"), Some(Greater));
}
#[test]
fn compare_version_2_to_1() {
  assert_eq!("2".ver_cmp("1"), Some(Greater));
}
#[test]
fn compare_version_1_to_2() {
  assert_eq!("1".ver_cmp("2"), Some(Less));
}
#[test]
fn compare_version_1_to_1_1() {
  assert_eq!("1".ver_cmp("1.1"), Some(Less));
}
#[test]
fn compare_version_1_to_1pl1() {
  assert_eq!("1".ver_cmp("1pl1"), Some(Less));
}
#[test]
fn compare_version_7_3_2_to_7_3ce_1() {
  // See https://github.com/voidlinux/void-packages/commit/7011dc83bbe6f3a25370c0fdb9e1fbf19ee1fe6b
  assert_eq!("7.3.2".ver_cmp("7.3ce.1"), None);
}

// COMPARE PARTS
#[test]
fn compare_component_alpha_to_alpha() {
  assert_eq!(Alpha.real_cmp(&Alpha), Some(Equal));
}
#[test]
fn compare_component_beta_to_alpha() {
  assert_eq!(Beta.real_cmp(&Alpha), Some(Greater));
  assert_eq!(Alpha.real_cmp(&Beta), Some(Less));
}
#[test]
fn compare_component_beta_to_beta() {
  assert_eq!(Beta.real_cmp(&Beta), Some(Equal));
}
#[test]
fn compare_component_pre_to_alpha() {
  assert_eq!(Pre.real_cmp(&Alpha), Some(Greater));
  assert_eq!(Alpha.real_cmp(&Pre), Some(Less));
}
#[test]
fn compare_component_pre_to_beta() {
  assert_eq!(Pre.real_cmp(&Beta), Some(Greater));
  assert_eq!(Beta.real_cmp(&Pre), Some(Less));
}
#[test]
fn compare_component_pre_to_pre() {
  assert_eq!(Pre.real_cmp(&Pre), Some(Equal));
}
#[test]
fn compare_component_rc_to_alpha() {
  assert_eq!(Rc.real_cmp(&Alpha), Some(Greater));
  assert_eq!(Alpha.real_cmp(&Rc), Some(Less));
}
#[test]
fn compare_component_rc_to_beta() {
  assert_eq!(Rc.real_cmp(&Beta), Some(Greater));
  assert_eq!(Beta.real_cmp(&Rc), Some(Less));
}
#[test]
fn compare_component_rc_to_pre() {
  assert_eq!(Rc.real_cmp(&Pre), Some(Greater));
  assert_eq!(Pre.real_cmp(&Rc), Some(Less));
}
#[test]
fn compare_component_rc_to_rc() {
  assert_eq!(Rc.real_cmp(&Rc), Some(Equal));
}
#[test]
fn compare_component_patchlevel_to_alpha() {
  assert_eq!(PatchLevel.real_cmp(&Alpha), Some(Greater));
  assert_eq!(Alpha.real_cmp(&PatchLevel), Some(Less));
}
#[test]
fn compare_component_patchlevel_to_beta() {
  assert_eq!(PatchLevel.real_cmp(&Beta), Some(Greater));
  assert_eq!(Beta.real_cmp(&PatchLevel), Some(Less));
}
#[test]
fn compare_component_patchlevel_to_pre() {
  assert_eq!(PatchLevel.real_cmp(&Pre), Some(Greater));
  assert_eq!(Pre.real_cmp(&PatchLevel), Some(Less));
}
#[test]
fn compare_component_patchlevel_to_rc() {
  assert_eq!(PatchLevel.real_cmp(&Rc), Some(Greater));
  assert_eq!(Rc.real_cmp(&PatchLevel), Some(Less));
}
#[test]
fn compare_component_patchlevel_to_patchlevel() {
  assert_eq!(PatchLevel.real_cmp(&PatchLevel), Some(Equal));
}
#[test]
fn compare_component_dot_to_alpha() {
  assert_eq!(DashOrDot.real_cmp(&Alpha), Some(Greater));
  assert_eq!(Alpha.real_cmp(&DashOrDot), Some(Less));
}
#[test]
fn compare_component_dot_to_beta() {
  assert_eq!(DashOrDot.real_cmp(&Beta), Some(Greater));
  assert_eq!(Beta.real_cmp(&DashOrDot), Some(Less));
}
#[test]
fn compare_component_dot_to_pre() {
  assert_eq!(DashOrDot.real_cmp(&Pre), Some(Greater));
  assert_eq!(Pre.real_cmp(&DashOrDot), Some(Less));
}
#[test]
fn compare_component_dot_to_rc() {
  assert_eq!(DashOrDot.real_cmp(&Rc), Some(Greater));
  assert_eq!(Rc.real_cmp(&DashOrDot), Some(Less));
}
#[test]
fn compare_component_dot_to_patchlevel() {
  assert_eq!(DashOrDot.real_cmp(&PatchLevel), None);
  assert_eq!(PatchLevel.real_cmp(&DashOrDot), None);
}
#[test]
fn compare_component_dot_to_dot() {
  assert_eq!(DashOrDot.real_cmp(&DashOrDot), Some(Equal));
}
#[test]
fn compare_component_end_to_alpha() {
  assert_eq!(End.real_cmp(&Alpha), Some(Greater));
  assert_eq!(Alpha.real_cmp(&End), Some(Less));
}
#[test]
fn compare_component_end_to_beta() {
  assert_eq!(End.real_cmp(&Beta), Some(Greater));
  assert_eq!(Beta.real_cmp(&End), Some(Less));
}
#[test]
fn compare_component_end_to_pre() {
  assert_eq!(End.real_cmp(&Pre), Some(Greater));
  assert_eq!(Pre.real_cmp(&End), Some(Less));
}
#[test]
fn compare_component_end_to_rc() {
  assert_eq!(End.real_cmp(&Rc), Some(Greater));
  assert_eq!(Rc.real_cmp(&End), Some(Less));
}
#[test]
fn compare_component_end_to_patchlevel() {
  assert_eq!(End.real_cmp(&PatchLevel), Some(Equal));
  assert_eq!(PatchLevel.real_cmp(&End), Some(Equal));
}
#[test]
fn compare_component_end_to_dot() {
  assert_eq!(End.real_cmp(&DashOrDot), Some(Equal));
  assert_eq!(DashOrDot.real_cmp(&End), Some(Equal));
}
#[test]
fn compare_component_end_to_end() {
  assert_eq!(End.real_cmp(&End), Some(Equal));
}
#[test]
fn compare_component_num_0_to_alpha() {
  assert_eq!(Num(0).real_cmp(&Alpha), Some(Greater));
  assert_eq!(Alpha.real_cmp(&Num(0)), Some(Less));
}
#[test]
fn compare_component_num_0_to_beta() {
  assert_eq!(Num(0).real_cmp(&Beta), Some(Greater));
  assert_eq!(Beta.real_cmp(&Num(0)), Some(Less));
}
#[test]
fn compare_component_num_0_to_pre() {
  assert_eq!(Num(0).real_cmp(&Pre), Some(Greater));
  assert_eq!(Pre.real_cmp(&Num(0)), Some(Less));
}
#[test]
fn compare_component_num_0_to_rc() {
  assert_eq!(Num(0).real_cmp(&Rc), Some(Greater));
  assert_eq!(Rc.real_cmp(&Num(0)), Some(Less));
}
#[test]
fn compare_component_num_0_to_patchlevel() {
  assert_eq!(Num(0).real_cmp(&PatchLevel), None);
  assert_eq!(PatchLevel.real_cmp(&Num(0)), None);
}
#[test]
fn compare_component_num_0_to_dot() {
  assert_eq!(Num(0).real_cmp(&DashOrDot), None);
  assert_eq!(DashOrDot.real_cmp(&Num(0)), None);
}
#[test]
fn compare_component_num_0_to_end() {
  assert_eq!(Num(0).real_cmp(&End), Some(Equal));
  assert_eq!(End.real_cmp(&Num(0)), Some(Equal));
}
#[test]
fn compare_component_num_0_to_num_0() {
  assert_eq!(Num(0).real_cmp(&Num(0)), Some(Equal));
}
#[test]
fn compare_component_char_a_to_alpha() {
  assert_eq!(Char('a').real_cmp(&Alpha), Some(Greater));
  assert_eq!(Alpha.real_cmp(&Char('a')), Some(Less));
}
#[test]
fn compare_component_char_a_to_beta() {
  assert_eq!(Char('a').real_cmp(&Beta), Some(Greater));
  assert_eq!(Beta.real_cmp(&Char('a')), Some(Less));
}
#[test]
fn compare_component_char_a_to_pre() {
  assert_eq!(Char('a').real_cmp(&Pre), Some(Greater));
  assert_eq!(Pre.real_cmp(&Char('a')), Some(Less));
}
#[test]
fn compare_component_char_a_to_rc() {
  assert_eq!(Char('a').real_cmp(&Rc), Some(Greater));
  assert_eq!(Rc.real_cmp(&Char('a')), Some(Less));
}
#[test]
fn compare_component_char_a_to_patchlevel() {
  assert_eq!(Char('a').real_cmp(&PatchLevel), Some(Greater));
  assert_eq!(PatchLevel.real_cmp(&Char('a')), Some(Less));
}
#[test]
fn compare_component_char_a_to_dot() {
  assert_eq!(Char('a').real_cmp(&DashOrDot), None);
  assert_eq!(DashOrDot.real_cmp(&Char('a')), None);
}
#[test]
fn compare_component_char_a_to_end() {
  assert_eq!(Char('a').real_cmp(&End), Some(Greater));
  assert_eq!(End.real_cmp(&Char('a')), Some(Less));
}
#[test]
fn compare_component_char_a_to_num_0() {
  assert_eq!(Char('a').real_cmp(&Num(0)), None);
  assert_eq!(Num(0).real_cmp(&Char('a')), None);
}
#[test]
fn compare_component_char_a_to_char_a() {
  assert_eq!(Char('a').real_cmp(&Char('a')), Some(Equal));
}
