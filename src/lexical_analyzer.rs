use std::collections::HashMap;
use crate::automata::dfa::DfaStateHandle;
use crate::automata::InputSymbol;
use crate::automata::labeled_dfa::{DfaLabel, LabeledDfa};
use crate::automata::nfa::NfaBuilder;
use crate::reader::Reader;
use crate::regex::Regex;

struct LexemeDescriptor<T> {
    pattern: Regex,
    lexeme_type: T,
}

pub struct Lexeme<T> {
    lexeme_type: T,
    contents: String,
}

pub struct LexemeIterator<'a, T, U>
where
    U: Reader<u8>,
    T: Clone,
{
    lexical_analyzer: &'a LexicalAnalyzer<T>,
    reader: &'a mut U,
}

impl<'a, T, U> LexemeIterator<'a, T, U>
where
    U: Reader<u8>,
    T: Clone,
{
    fn new(lexical_analyzer: &'a LexicalAnalyzer<T>, reader: &'a mut U) -> Self
    {
        Self { lexical_analyzer, reader }
    }
}

impl<'a, T, U> Iterator for LexemeIterator<'a, T, U>
where
    U: Reader<u8>,
    T: Clone,
{
    type Item = Lexeme<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexical_analyzer.collect_next_lexeme(self.reader)
    }
}

// TODO doc: maximum of 256 symbols
struct LexicalAnalyzer<T>
where
    T: Clone,
{
    labeled_dfa: LabeledDfa,
    lexeme_types_map: HashMap<DfaLabel, T>,
    dead_state: DfaStateHandle,
}

// Earlier lexeme descriptors are prioritized
// TODO doc maximum number of patterns is 255 (not 256!)
impl<T> LexicalAnalyzer<T>
where
    T: Clone,
{
    pub fn new(lexeme_descriptors: Vec<LexemeDescriptor<T>>) -> LexicalAnalyzer<T> {
        let mut nfa_builder = NfaBuilder::new(u8::MAX as u16);
        let global_start_state = nfa_builder.new_state();
        let mut lexeme_types_for_dfa_labels = HashMap::new();

        let mut nfa_state_to_label_map = HashMap::new();
        for (pattern_index, LexemeDescriptor { pattern, lexeme_type })
        in lexeme_descriptors.iter().enumerate() {

            // Create DFA label for pattern
            // We add 1 to the pattern index, to reserve label 0 for "no pattern associated"
            let label = DfaLabel((pattern_index + 1) as u8); // Possible type confusion
            lexeme_types_for_dfa_labels.insert(label, lexeme_type.clone());

            // Build pattern into the nfa
            let (pattern_start_state, pattern_end_state) =
                pattern.build_into_nfa(&mut nfa_builder);
            nfa_builder.link(global_start_state, pattern_start_state, None);
            nfa_state_to_label_map.insert(pattern_end_state, label);
        }

        // Compile the NFA to a DFA
        let nfa = nfa_builder.build(global_start_state);
        let (dfa, dfa_to_nfa_states_map)
            = nfa.compile_to_dfa();

        // Label DFA states according to the first pattern associated with them
        let mut unoptimized_labeled_dfa = LabeledDfa::new(dfa);
        for (&dfa_state, associated_nfa_states)
        in dfa_to_nfa_states_map.iter() {
            let optional_label = associated_nfa_states
                .iter()
                .map(|nfa_state| nfa_state_to_label_map.get(nfa_state))
                .flatten()
                .filter(|x| x.0 != 0)
                .min(); // We pick the minimum label to prioritize earlier lexeme descriptions
            if let Some(&label) = optional_label {
                unoptimized_labeled_dfa.label(dfa_state, label)
            }
        }

        // Optimize the DFA (minimize its number of states) and locate dead state
        let labeled_dfa = unoptimized_labeled_dfa.minimize();
        let dead_state = labeled_dfa.dfa.locate_dead_state().expect(
            "A minimized DFA for regex-based lexical analyzer is expected to have a dead state"
        );

        LexicalAnalyzer { labeled_dfa, lexeme_types_map: lexeme_types_for_dfa_labels, dead_state }
    }

    pub fn parse<'a>(&'a self, reader: &'a mut impl Reader<u8>)
        -> impl Iterator<Item=Lexeme<T>> + 'a
    {
        LexemeIterator::new(self, reader)
    }

    fn identify_next_lexeme(&self, reader: &mut impl Reader<u8>) -> Option<T> {
        // TODO error recovery
        let mut recent_lexeme_type: Option<T> = None;
        let mut current_state = self.labeled_dfa.dfa.initial_state;
        while current_state != self.dead_state {
            let current_state_label = self.labeled_dfa.get_label(current_state);
            if let Some(lexeme_type) = self.lexeme_types_map.get(&current_state_label) {
                recent_lexeme_type = Some(lexeme_type.clone());
                reader.set_tail();
            }
            let next_input_symbol = InputSymbol { id: reader.read_next()? as u16 };
            current_state = self.labeled_dfa.dfa.step(current_state, next_input_symbol)
        }
        recent_lexeme_type
    }

    fn collect_next_lexeme(&self, reader: &mut impl Reader<u8>) -> Option<Lexeme<T>> {
        let lexeme_type = self.identify_next_lexeme(reader)?;
        let contents = String::from_utf8(reader.get_sequence().collect()).expect(
            "Tokens from lexically-analyzed Reader<u8> are expected to be UTF-8 encoded"
        );
        let lexeme = Lexeme { lexeme_type, contents };
        reader.restart_from_tail();
        Some(lexeme)
    }
}

// TODO add tests