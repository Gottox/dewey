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

#[derive(Clone)]
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
pub struct Version {
  components: Vec<Component>,
  version: String,
}

impl Version {
  /// Returns a Result containing either a Version representation of the
  /// string or - if unparsable - an Err instance.
  pub fn new(version: &str) -> Result<Version, &'static str> {
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

  /// Extracts a string slice containing the the original version number
  pub fn as_str(&self) -> &str {
    self.version.as_str()
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
  let v = Version::new("1").ok().unwrap();
  assert_eq!(v.components[0].v, 1);
}

#[test]
fn compare_letters() {
  assert!(Version::new("A") == Version::new("a"));
  assert!(Version::new("a") < Version::new("b"));
  assert!(Version::new("aa") < Version::new("b"));
}

#[test]
fn compare_equivalent() {
  assert!(Version::new("1") == Version::new("1"));
  assert!(Version::new("1") == Version::new("1.0"));
  assert!(Version::new("1") == Version::new("1pl0"));
}

#[test]
fn compare_smaller() {
  assert!(Version::new("1") > Version::new("0"));
  assert!(Version::new("1") > Version::new("0.0.1"));
  assert!(Version::new("1") > Version::new("1pre1"));
  assert!(Version::new("1") > Version::new("1rc1"));
  assert!(Version::new("1") > Version::new("1alpha"));
  assert!(Version::new("1") > Version::new("1alpha1"));
  assert!(Version::new("1") > Version::new("1beta1"));
}

#[test]
fn compare_greater() {
  assert!(Version::new("1") < Version::new("2"));
  assert!(Version::new("1") < Version::new("1.1"));
  assert!(Version::new("1") < Version::new("1pl1"));
}

#[test]
fn compare_invalid() {
  assert!(Version::new("1a")
            .partial_cmp(&Version::new("1.0"))
            .is_none());
}
