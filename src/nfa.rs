use std::hashmap::HashSet;

#[deriving(Eq)]
pub enum CharacterClass {
  ValidChars(HashSet<~char>),
  InvalidChars(HashSet<~char>)
}

impl CharacterClass {
  pub fn valid(string: &str) -> CharacterClass {
    ValidChars(CharacterClass::str_to_set(string))
  }

  pub fn valid_char(char: char) -> CharacterClass {
    ValidChars(CharacterClass::char_to_set(char))
  }

  pub fn invalid(string: &str) -> CharacterClass {
    InvalidChars(CharacterClass::str_to_set(string))
  }

  pub fn invalid_char(char: char) -> CharacterClass {
    InvalidChars(CharacterClass::char_to_set(char))
  }

  fn char_to_set(char: char) -> HashSet<~char> {
    let mut set = HashSet::new();
    set.insert(~char);
    set
  }

  fn str_to_set(string: &str) -> HashSet<~char> {
    let mut set = HashSet::new();
    for char in string.chars() {
      set.insert(~char);
    }
    set
  }
}

struct State<T> {
  index: uint,
  chars: CharacterClass,
  next_states: ~[uint],
  acceptance: bool,
  metadata: Option<T>
}

impl<T> Eq for State<T> {
  fn eq(&self, other: &State<T>) -> bool {
    self.index == other.index
  }
}

impl<T> State<T> {
  pub fn new(index: uint, chars: CharacterClass) -> State<T> {
    State{ index: index, chars: chars, next_states: ~[], acceptance: false, metadata: None }
  }
}

pub struct NFA<T> {
  states: ~[State<T>]
}

impl<T> NFA<T> {
  pub fn new() -> NFA<T> {
    let root = State::new(0, CharacterClass::valid(""));
    NFA{ states: ~[root] }
  }

  pub fn process<'a>(&'a self, string: &str, sort: |a: &[uint], b: &[uint]| -> Ordering) -> Result<&'a State<T>, ~str> {
    let mut current = ~[~[0]];

    for char in string.chars() {
      let next_traces = self.process_char(current, &char);

      if next_traces.is_empty() {
        return Err("Couldn't process " + string);
      }

      current = next_traces;
    }

    let mut returned = current.iter().filter(|trace| {
      self.get(*trace.last()).acceptance
    }).map(|trace| (*trace).as_slice()).to_owned_vec();

    if returned.is_empty() {
      Err(~"The string was exhausted before reaching an acceptance state")
    } else {
      returned.sort_by(|a,b| sort(*a, *b));
      Ok(self.get(*returned.last().last()))
    }
  }

  fn process_char<'a>(&'a self, traces: ~[~[uint]], char: &char) -> ~[~[uint]] {
    let mut returned = ~[];

    for trace in traces.iter() {
      let state = self.get(*trace.last());
      for index in state.next_states.iter() {
        let state = self.get(*index);
        match state.chars {
          ValidChars(ref valid) => if valid.contains(&~*char) {
            let mut new_trace = trace.clone();
            new_trace.push(state.index);
            returned.push(new_trace);
          },
          InvalidChars(ref invalid) => if !invalid.contains(&~*char) {
            let mut new_trace = trace.clone();
            new_trace.push(state.index);
            returned.push(new_trace);
          }
        }
      }
    }

    returned
  }

  pub fn get<'a>(&'a self, state: uint) -> &'a State<T> {
    &self.states[state]
  }

  pub fn get_mut<'a>(&'a mut self, state: uint) -> &'a mut State<T> {
    &mut self.states[state]
  }

  pub fn put(&mut self, index: uint, chars: CharacterClass) -> uint {
    {
      let state = self.get(index);

      for index in state.next_states.iter() {
        let state = self.get(*index);
        if state.chars == chars {
          return *index;
        }
      }
    }

    let state = self.new_state(chars);
    self.get_mut(index).next_states.push(state);
    state
  }

  pub fn put_state(&mut self, index: uint, child: uint) {
    self.get_mut(index).next_states.push(child);
  }

  pub fn acceptance(&mut self, index: uint) {
    self.get_mut(index).acceptance = true;
  }

  pub fn metadata(&mut self, index: uint, metadata: T) {
    self.get_mut(index).metadata = Some(metadata);
  }

  fn new_state(&mut self, chars: CharacterClass) -> uint {
    let index = self.states.len();
    let state = State::new(index, chars);
    self.states.push(state);
    index
  }
}

#[test]
fn basic_test() {
  let mut nfa = NFA::<()>::new();
  let a = nfa.put(0, CharacterClass::valid("h"));
  let b = nfa.put(a, CharacterClass::valid("e"));
  let c = nfa.put(b, CharacterClass::valid("l"));
  let d = nfa.put(c, CharacterClass::valid("l"));
  let e = nfa.put(d, CharacterClass::valid("o"));
  nfa.acceptance(e);

  let states = nfa.process("hello", |a,b| a.len().cmp(&b.len()));

  assert!(states.unwrap() == nfa.get(e), "You didn't get the right final state");
}

#[test]
fn multiple_solutions() {
  let mut nfa = NFA::<()>::new();
  let a1 = nfa.put(0, CharacterClass::valid("n"));
  let b1 = nfa.put(a1, CharacterClass::valid("e"));
  let c1 = nfa.put(b1, CharacterClass::valid("w"));
  nfa.acceptance(c1);

  let a2 = nfa.put(0, CharacterClass::invalid(""));
  let b2 = nfa.put(a2, CharacterClass::invalid(""));
  let c2 = nfa.put(b2, CharacterClass::invalid(""));
  nfa.acceptance(c2);

  let states = nfa.process("new", |a,b| a.len().cmp(&b.len()));

  assert!(states.unwrap() == nfa.get(c2), "The two states were not found");
}

#[test]
fn multiple_paths() {
  let mut nfa = NFA::<()>::new();
  let a = nfa.put(0, CharacterClass::valid("t"));   // t
  let b1 = nfa.put(a, CharacterClass::valid("h"));  // th
  let c1 = nfa.put(b1, CharacterClass::valid("o")); // tho
  let d1 = nfa.put(c1, CharacterClass::valid("m")); // thom
  let e1 = nfa.put(d1, CharacterClass::valid("a")); // thoma
  let f1 = nfa.put(e1, CharacterClass::valid("s")); // thomas

  let b2 = nfa.put(a, CharacterClass::valid("o"));  // to
  let c2 = nfa.put(b2, CharacterClass::valid("m")); // tom

  nfa.acceptance(f1);
  nfa.acceptance(c2);

  let thomas = nfa.process("thomas", |a,b| a.len().cmp(&b.len()));
  let tom = nfa.process("tom", |a,b| a.len().cmp(&b.len()));
  let thom = nfa.process("thom", |a,b| a.len().cmp(&b.len()));
  let nope = nfa.process("nope", |a,b| a.len().cmp(&b.len()));

  assert!(thomas.unwrap() == nfa.get(f1), "thomas was parsed correctly");
  assert!(tom.unwrap() == nfa.get(c2), "tom was parsed correctly");
  assert!(thom.is_err(), "thom didn't reach an acceptance state");
  assert!(nope.is_err(), "nope wasn't parsed");
}

#[test]
fn repetitions() {
  let mut nfa = NFA::<()>::new();
  let a = nfa.put(0, CharacterClass::valid("p"));   // p
  let b = nfa.put(a, CharacterClass::valid("o"));   // po
  let c = nfa.put(b, CharacterClass::valid("s"));   // pos
  let d = nfa.put(c, CharacterClass::valid("t"));   // post
  let e = nfa.put(d, CharacterClass::valid("s"));   // posts
  let f = nfa.put(e, CharacterClass::valid("/"));   // posts/
  let g = nfa.put(f, CharacterClass::invalid("/")); // posts/[^/]
  nfa.put_state(g, g);

  nfa.acceptance(g);

  let post = nfa.process("posts/1", |a,b| a.len().cmp(&b.len()));
  let new_post = nfa.process("posts/new", |a,b| a.len().cmp(&b.len()));
  let invalid = nfa.process("posts/", |a,b| a.len().cmp(&b.len()));

  assert!(post.unwrap() == nfa.get(g), "posts/1 was parsed");
  assert!(new_post.unwrap() == nfa.get(g), "posts/new was parsed");
  assert!(invalid.is_err(), "posts/ was invalid");
}

#[test]
fn repetitions_with_ambiguous() {
  let mut nfa = NFA::<()>::new();
  let a  = nfa.put(0, CharacterClass::valid("p"));   // p
  let b  = nfa.put(a, CharacterClass::valid("o"));   // po
  let c  = nfa.put(b, CharacterClass::valid("s"));   // pos
  let d  = nfa.put(c, CharacterClass::valid("t"));   // post
  let e  = nfa.put(d, CharacterClass::valid("s"));   // posts
  let f  = nfa.put(e, CharacterClass::valid("/"));   // posts/
  let g1 = nfa.put(f, CharacterClass::invalid("/")); // posts/[^/]
  let g2 = nfa.put(f, CharacterClass::valid("n"));   // posts/n
  let h2 = nfa.put(g2, CharacterClass::valid("e"));  // posts/ne
  let i2 = nfa.put(h2, CharacterClass::valid("w"));  // posts/new

  nfa.put_state(g1, g1);

  nfa.acceptance(g1);
  nfa.acceptance(i2);

  let post = nfa.process("posts/1", |a,b| a.len().cmp(&b.len()));
  let ambiguous = nfa.process("posts/new", |a,b| a.len().cmp(&b.len()));
  let invalid = nfa.process("posts/", |a,b| a.len().cmp(&b.len()));

  assert!(post.unwrap() == nfa.get(g1), "posts/1 was parsed");
  assert!(ambiguous.unwrap() == nfa.get(i2), "posts/new was ambiguous");
  assert!(invalid.is_err(), "posts/ was invalid");
}
