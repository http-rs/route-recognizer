use nfa::NFA;
use nfa::CharacterClass;
use std::hashmap::HashMap;
mod nfa;

#[deriving(Clone)]
struct Metadata {
  statics: uint,
  dynamics: uint,
  stars: uint,
  param_names: ~[~str]
}

impl Metadata {
  pub fn new() -> Metadata {
    Metadata{ statics: 0, dynamics: 0, stars: 0, param_names: ~[] }
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

pub struct Match<T> {
  handler: T,
  params: HashMap<~str, ~str>
}

impl<T> Match<T> {
  pub fn new(handler: T, params: HashMap<~str, ~str>) -> Match<T> {
    Match{ handler: handler, params: params }
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
        metadata.param_names.push(segment.slice_from(1).to_owned());
      } else {
        state = process_static_segment(segment, nfa, state);
        metadata.statics += 1;
      }
    }

    nfa.acceptance(state);
    nfa.metadata(state, metadata);
    self.handlers.insert(state, dest);
  }

  pub fn recognize<'a>(&'a self, mut path: &str) -> Result<Match<&'a T>, ~str> {
    if path.char_at(0) == '/' {
      path = path.slice_from(1);
    }

    let nfa = &self.nfa;
    let result = nfa.process(path, |a,b| nfa.get(*a.last()).metadata.get_ref().cmp(nfa.get(*b.last()).metadata.get_ref()));

    match result {
      Ok(nfa_match) => {
        let mut map = HashMap::new();
        let state = &nfa.get(nfa_match.state);
        let metadata = state.metadata.get_ref();
        let param_names = metadata.param_names.clone();

        for (i, capture) in nfa_match.captures.iter().enumerate() {
          map.insert(param_names[i].to_owned(), capture.to_owned());
        }

        let handler = self.handlers.get(&nfa_match.state);
        Ok(Match::new(handler, map))
      },
      Err(str) => Err(str)
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
  nfa.start_capture(state);
  nfa.end_capture(state);

  state
}

#[test]
fn basic_router() {
  let mut router = Router::new();

  router.add("/thomas", ~"Thomas");
  router.add("/tom", ~"Tom");
  router.add("/wycats", ~"Yehuda");

  let m = router.recognize("/thomas").unwrap();

  assert_eq!(*m.handler, ~"Thomas");
  assert_eq!(m.params, HashMap::new());
}

#[test]
fn ambiguous_router() {
  let mut router = Router::new();

  router.add("/posts/new", ~"new");
  router.add("/posts/:id", ~"id");

  let id = router.recognize("/posts/1").unwrap();

  assert_eq!(*id.handler, ~"id");
  assert_eq!(id.params, params("id", "1"));

  let new = router.recognize("/posts/new").unwrap();
  assert_eq!(*new.handler, ~"new");
  assert_eq!(new.params, HashMap::new());
}


#[test]
fn ambiguous_router_b() {
  let mut router = Router::new();

  router.add("/posts/:id", ~"id");
  router.add("/posts/new", ~"new");

  let id = router.recognize("/posts/1").unwrap();

  assert_eq!(*id.handler, ~"id");
  assert_eq!(id.params, params("id", "1"));

  let new = router.recognize("/posts/new").unwrap();
  assert_eq!(*new.handler, ~"new");
  assert_eq!(new.params, HashMap::new());
}


#[test]
fn multiple_params() {
  let mut router = Router::new();

  router.add("/posts/:post_id/comments/:id", ~"comment");
  router.add("/posts/:post_id/comments", ~"comments");

  let com = router.recognize("/posts/12/comments/100").unwrap();
  let coms = router.recognize("/posts/12/comments").unwrap();

  assert_eq!(*com.handler, ~"comment");
  assert_eq!(com.params, two_params("post_id", "12", "id", "100"));

  assert_eq!(*coms.handler, ~"comments");
  assert_eq!(coms.params, params("post_id", "12"));
}

fn params(key: &str, val: &str) -> HashMap<~str, ~str> {
  let mut map = HashMap::new();
  map.insert(key.to_owned(), val.to_owned());
  map
}

fn two_params(k1: &str, v1: &str, k2: &str, v2: &str) -> HashMap<~str, ~str> {
  let mut map = HashMap::new();
  map.insert(k1.to_owned(), v1.to_owned());
  map.insert(k2.to_owned(), v2.to_owned());
  map
}
