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

    for char in route.chars() {
      state = nfa.put(state, CharacterClass::valid_char(char));
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
