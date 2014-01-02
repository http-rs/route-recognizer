use nfa::NFA;
use nfa::CharacterClass;
use std::hashmap::HashMap;
mod nfa;

pub enum Handler {
  StringHandler(~str)
}

struct Metadata {
  statics: uint,
  dynamics: uint,
  stars: uint
}

impl Metadata {
  pub fn new() -> Metadata {
    Metadata{ statics: 0, dynamics: 0, stars: 0 }
  }
}

impl TotalOrd for Metadata {
  fn cmp(&self, other: &Metadata) -> Ordering {
    if self.stars > other.stars {
      Less
    } else if self.stars < other.stars {
      Greater
    } else if self.dynamics > other.dynamics {
      Less
    } else if self.dynamics < other.dynamics {
      Greater
    } else if self.statics > other.statics {
      Less
    } else if self.statics < other.statics {
      Greater
    } else {
      Equal
    }
  }
}

impl TotalEq for Metadata {
  fn equals(&self, other: &Metadata) -> bool {
    self.statics == other.statics && self.dynamics == other.dynamics && self.stars == other.stars
  }
}

pub struct Router<T> {
  nfa: NFA<Metadata>,
  handlers: HashMap<uint, T>
}

impl<T> Router<T> {
  pub fn new() -> Router<T> {
    Router{ nfa: NFA::new(), handlers: HashMap::new() }
  }

  pub fn add(&mut self, mut route: &str, dest: T) {
    if route.char_at(0) == '/' {
      route = route.slice_from(1);
    }

    let nfa = &mut self.nfa;
    let mut state = 0;
    let mut metadata = Metadata::new();

    for (i, segment) in route.split('/').enumerate() {
      if i > 0 { state = nfa.put(state, CharacterClass::valid_char('/')); }

      if segment.char_at(0) == ':' {
        state = process_dynamic_segment(nfa, state);
        metadata.dynamics += 1;
      } else {
        state = process_static_segment(segment, nfa, state);
        metadata.statics += 1;
      }
    }

    nfa.acceptance(state);
    nfa.metadata(state, metadata);
    self.handlers.insert(state, dest);
  }

  pub fn recognize<'a>(&'a self, mut path: &str) -> Result<&'a T, ~str> {
    if path.char_at(0) == '/' {
      path = path.slice_from(1);
    }

    let states = self.nfa.process(path);

    match states {
      Err(str) => Err(str),
      Ok(mut states) => {
        states.sort_by(|a, b| a.metadata.get_ref().cmp(b.metadata.get_ref()));
        Ok(self.handlers.get(&states.last().index))
      }
    }
  }
}

fn process_static_segment<T>(segment: &str, nfa: &mut NFA<T>, mut state: uint) -> uint {
  for char in segment.chars() {
    state = nfa.put(state, CharacterClass::valid_char(char));
  }

  state
}

fn process_dynamic_segment<T>(nfa: &mut NFA<T>, mut state: uint) -> uint {
  state = nfa.put(state, CharacterClass::invalid_char('/'));
  nfa.put_state(state, state);

  state
}

#[test]
fn basic_router() {
  let mut router = Router::<Handler>::new();

  router.add("/thomas", StringHandler(~"Thomas"));
  router.add("/tom", StringHandler(~"Tom"));
  router.add("/wycats", StringHandler(~"Yehuda"));

  match *router.recognize("/thomas").unwrap() {
    StringHandler(ref str) => assert!(str == &~"Thomas", "/thomas matched")
  }
}

#[test]
fn ambiguous_router() {
  let mut router = Router::<Handler>::new();

  router.add("/posts/new", StringHandler(~"new"));
  router.add("/posts/:id", StringHandler(~"id"));

  match *router.recognize("/posts/1").unwrap() {
    StringHandler(ref str) => assert!(str == &~"id", "/posts/1 matched")
  }

  match *router.recognize("/posts/new").unwrap() {
    StringHandler(ref str) => assert!(str == &~"new", "/posts/new matched")
  }
}

#[test]
fn ambiguous_router_b() {
  let mut router = Router::<Handler>::new();

  router.add("/posts/:id", StringHandler(~"id"));
  router.add("/posts/new", StringHandler(~"new"));

  match *router.recognize("/posts/1").unwrap() {
    StringHandler(ref str) => assert!(str == &~"id", "/posts/1 matched")
  }

  match *router.recognize("/posts/new").unwrap() {
    StringHandler(ref str) => assert!(str == &~"new", "/posts/new matched")
  }
}
