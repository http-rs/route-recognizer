use nfa::NFA;
use nfa::CharacterClass;
use std::hashmap::HashMap;
mod nfa;

pub enum Handler {
  StringHandler(~str)
}

struct Router {
  nfa: NFA,
  handlers: HashMap<uint, Handler>
}

impl Router {
  pub fn new() -> Router {
    Router{ nfa: NFA::new(), handlers: HashMap::new() }
  }

  pub fn add(&mut self, mut route: &str, dest: Handler) {
    if route.char_at(0) == '/' {
      route = route.slice_from(1);
    }

    let nfa = &mut self.nfa;
    let mut state = 0;

    for (i, segment) in route.split('/').enumerate() {
      if i > 0 { state = nfa.put(state, CharacterClass::valid_char('/')); }

      if segment.char_at(0) == ':' {
        state = process_dynamic_segment(nfa, state);
      } else {
        state = process_static_segment(segment, nfa, state);
      }
    }

    nfa.acceptance(state);
    self.handlers.insert(state, dest);
  }

  pub fn recognize<'a>(&'a self, mut path: &str) -> Result<&'a Handler, ~str> {
    if path.char_at(0) == '/' {
      path = path.slice_from(1);
    }

    let states = self.nfa.process(path);

    match states {
      Err(str) => Err(str),
      Ok(states) => Ok(self.handlers.get(&states[0]))
    }
  }
}

fn process_static_segment(segment: &str, nfa: &mut NFA, mut state: uint) -> uint {
  for char in segment.chars() {
    state = nfa.put(state, CharacterClass::valid_char(char));
  }

  state
}

fn process_dynamic_segment(nfa: &mut NFA, mut state: uint) -> uint {
  state = nfa.put(state, CharacterClass::invalid_char('/'));
  nfa.put_state(state, state);

  state
}

#[test]
fn basic_router() {
  let mut router = Router::new();

  router.add("/thomas", StringHandler(~"Thomas"));
  router.add("/tom", StringHandler(~"Tom"));
  router.add("/wycats", StringHandler(~"Yehuda"));

  match *router.recognize("/thomas").unwrap() {
    StringHandler(ref str) => assert!(str == &~"Thomas", "/thomas matched")
  }
}

#[test]
fn ambiguous_router() {
  let mut router = Router::new();

  router.add("/posts/new", StringHandler(~"new"));
  router.add("/posts/:id", StringHandler(~"id"));

  match *router.recognize("/posts/1").unwrap() {
    StringHandler(ref str) => assert!(str == &~"id", "/posts/1 matched")
  }
}
