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

struct State {
  index: uint,
  chars: CharacterClass,
  next_states: ~[uint],
  acceptance: bool
}

impl State {
  pub fn new(index: uint, chars: CharacterClass) -> State {
    State{ index: index, chars: chars, next_states: ~[], acceptance: false }
  }
}

pub struct NFA {
  states: ~[State]
}

impl NFA {
  pub fn new() -> NFA {
    let root = State::new(0, CharacterClass::valid(""));
    NFA{ states: ~[root] }
  }

  pub fn process<'a>(&'a self, string: &str) -> Result<~[uint], ~str> {
    let mut current = ~[self.get(0)];

    for char in string.chars() {
      let next_states = self.process_char(current, &char);

      if next_states.is_empty() {
        return Err("Couldn't process " + string);
      }

      current = next_states;
    }

    let returned = current.iter().filter_map(|&state| {
      if state.acceptance { Some(state.index) } else { None }
    }).to_owned_vec();

    if returned.is_empty() {
      Err(~"The string was exhausted before reaching an acceptance state")
    } else {
      Ok(returned)
    }
  }

  fn process_char<'a>(&'a self, states: ~[&State], char: &char) -> ~[&'a State] {
    let mut returned = ~[];

    for state in states.iter() {
      for index in state.next_states.iter() {
        let state = self.get(*index);
        match state.chars {
          ValidChars(ref valid) => if valid.contains(&~*char) { returned.push(state); },
          InvalidChars(ref invalid) => if !invalid.contains(&~*char) { returned.push(state); }
        }
      }
    }

    returned
  }

  pub fn get<'a>(&'a self, state: uint) -> &'a State {
    &self.states[state]
  }

  pub fn get_mut<'a>(&'a mut self, state: uint) -> &'a mut State {
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

  pub fn acceptance(&mut self, index: uint) {
    self.get_mut(index).acceptance = true;
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
  let mut nfa = NFA::new();
  let a = nfa.put(0, CharacterClass::valid("h"));
  let b = nfa.put(a, CharacterClass::valid("e"));
  let c = nfa.put(b, CharacterClass::valid("l"));
  let d = nfa.put(c, CharacterClass::valid("l"));
  let e = nfa.put(d, CharacterClass::valid("o"));
  nfa.acceptance(e);

  let states = nfa.process("hello");

  assert!(states.unwrap() == ~[e], "You didn't get the right final state");
}

#[test]
fn multiple_solutions() {
  let mut nfa = NFA::new();
  let a1 = nfa.put(0, CharacterClass::valid("n"));
  let b1 = nfa.put(a1, CharacterClass::valid("e"));
  let c1 = nfa.put(b1, CharacterClass::valid("w"));
  nfa.acceptance(c1);

  let a2 = nfa.put(0, CharacterClass::invalid(""));
  let b2 = nfa.put(a2, CharacterClass::invalid(""));
  let c2 = nfa.put(b2, CharacterClass::invalid(""));
  nfa.acceptance(c2);

  let states = nfa.process("new");

  assert!(states.unwrap() == ~[c1, c2], "The two states were not found");
}

#[test]
fn multiple_paths() {
  let mut nfa = NFA::new();
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

  let thomas = nfa.process("thomas");
  let tom = nfa.process("tom");
  let thom = nfa.process("thom");
  let nope = nfa.process("nope");

  assert!(thomas.unwrap() == ~[f1], "thomas was parsed correctly");
  assert!(tom.unwrap() == ~[c2], "tom was parsed correctly");
  assert!(thom.is_err(), "thom didn't reach an acceptance state");
  assert!(nope.is_err(), "nope wasn't parsed");
}
