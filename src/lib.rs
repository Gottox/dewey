//! This crate parses version strings using the dewey standard that is used in
//! NetBSD and Void Linux.

#![warn(missing_docs)]

#[macro_use]
extern crate static_map;
#[macro_use]
extern crate static_map_macros;

use static_map::Map;
use std::cmp::Ordering;
use std::ascii::AsciiExt;
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Debug)]
enum ComponentType {
  Revision,
  Alpha,
  Beta,
  Pre,
  Rc,
  PatchLevel,
  Dot,
  Number,
  Letter,
  Unknown,
}

#[derive(Clone, Debug)]
struct Component {
  t: ComponentType,
  v: i64,
}

static COMPARABLE: &[(ComponentType, ComponentType)] =
  &[(ComponentType::Alpha, ComponentType::Beta),
    (ComponentType::Beta, ComponentType::Dot),
    (ComponentType::Pre, ComponentType::Beta),
    (ComponentType::Pre, ComponentType::Dot)];

static MODIFIERS: Map<&'static str, Component> = static_map! {
    Default: Component{t:ComponentType::Unknown, v:0},
    "_" => Component{t:ComponentType::Revision, v:-10},
    "alpha" => Component{t:ComponentType::Alpha, v:-3},
    "beta" => Component{t:ComponentType::Beta, v:-2},
    "pre" => Component{t:ComponentType::Pre, v:-1},
    "rc" => Component{t:ComponentType::Rc, v:-1},
    "pl" => Component{t:ComponentType::PatchLevel, v:0},
    "." => Component{t:ComponentType::Dot, v:0}
};

/// An abstract definition of a version definition.
#[derive(Debug)]
pub struct Version {
  components: Vec<Component>,
  version: String,
}

impl FromStr for Version {
  type Err = &'static str;

  fn from_str(version: &str) -> Result<Self, Self::Err> {
    let chunkstr = version.to_lowercase();
    let mut chunk = chunkstr.as_str();
    let mut components = Vec::new();

    while chunk.len() > 0 {
      let split_at = if chunk.chars().nth(0).unwrap().is_digit(10) {
        chunk
          .chars()
          .position(|x| !x.is_digit(10))
          .unwrap_or(chunk.len())
      } else if chunk.chars().nth(0).unwrap().is_alphabetic() {
        chunk
          .chars()
          .position(|x| !x.is_alphabetic())
          .unwrap_or(chunk.len())
      } else {
        1
      };

      let (left, right) = chunk.split_at(split_at);
      if left.chars().nth(0).unwrap().is_digit(10) {
        components.push(Component {
                          t: ComponentType::Number,
                          v: left.parse().unwrap(),
                        })
      } else if let Some(m) = MODIFIERS.get(left) {
        components.push(m.clone())
      } else {
        if !left.chars().all(|c| c.is_ascii() && c.is_alphabetic()) {
          return Err("Version contains invalid characters");
        }
        components.extend(left
                            .chars()
                            .map(|c| {
                                   Component {
                                     t: ComponentType::Letter,
                                     v: c as i64 - 'a' as i64,
                                   }
                                 }));
      }

      chunk = right;
    }
    Ok(Version {
         version: version.to_string(),
         components: components,
       })
  }
}

impl ToString for Version {
  fn to_string(&self) -> String {
    self.version.clone()
  }
}

impl PartialEq for Version {
  fn eq(&self, other: &Version) -> bool {
    self
      .partial_cmp(other)
      .map(|x| x == Ordering::Equal)
      .unwrap_or(false)
  }
}

impl PartialOrd for Version {
  fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
    let mut si = self.components.iter();
    let mut oi = other.components.iter();
    for (s, o) in si.by_ref().zip(oi.by_ref()) {
      // Uncomparable Combinations
      if s.t != o.t &&
         !COMPARABLE
            .iter()
            .any(|&(t1, t2)| {
                   (t1 == s.t && t2 == o.t) || (t1 == o.t && t2 == s.t)
                 }) {
        return None;
      }
      if s.v > o.v {
        return Some(Ordering::Greater);
      } else if s.v < o.v {
        return Some(Ordering::Less);
      }
    }
    // One of the iterators is empty.
    for v in si.map(|x| x.v).chain(oi.map(|x| x.v * -1)) {
      if v > 0 {
        return Some(Ordering::Greater);
      } else if v < 0 {
        return Some(Ordering::Less);
      }
    }
    Some(Ordering::Equal)
  }
}

#[test]
fn parse_simple_number() {
  let v = Version::from_str("1").ok().unwrap();
  assert_eq!(v.components[0].v, 1);
}

#[test]
fn compare_letters() {
  assert!("A".parse::<Version>() == "a".parse::<Version>());

  assert!("A".parse::<Version>() == "a".parse::<Version>());
  assert!("a".parse::<Version>() < "b".parse::<Version>());
  assert!("aa".parse::<Version>() < "b".parse::<Version>());
}

#[test]
fn compare_equivalent() {
  assert!("1".parse::<Version>() == "1".parse::<Version>());
  assert!("1".parse::<Version>() == "1.0".parse::<Version>());
  assert!("1".parse::<Version>() == "1pl0".parse::<Version>());
}

#[test]
fn compare_smaller() {
  assert!("1".parse::<Version>() > "0".parse::<Version>());
  assert!("1".parse::<Version>() > "0.0.1".parse::<Version>());
  assert!("1".parse::<Version>() > "1pre1".parse::<Version>());
  assert!("1".parse::<Version>() > "1rc1".parse::<Version>());
  assert!("1".parse::<Version>() > "1alpha".parse::<Version>());
  assert!("1".parse::<Version>() > "1alpha1".parse::<Version>());
  assert!("1".parse::<Version>() > "1beta1".parse::<Version>());
}

#[test]
fn compare_greater() {
  assert!("1".parse::<Version>() < "2".parse::<Version>());
  assert!("1".parse::<Version>() < "1.1".parse::<Version>());
  assert!("1".parse::<Version>() < "1pl1".parse::<Version>());
}

#[test]
fn compare_invalid() {
  assert!("1a"
            .parse::<Version>()
            .partial_cmp(&"1.0".parse::<Version>())
            .is_none());
}
