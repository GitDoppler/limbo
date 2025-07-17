use turso_macros::Opcode;

#[derive(Debug, Clone, PartialEq)]
pub struct OpCodeDescription {
    pub name: &'static str,
    pub description: &'static str,
}

// Test tuple structs (unnamed fields)
#[allow(dead_code)]
#[derive(Debug, Clone, Opcode)]
pub enum TupleInstruction {
    #[description = "Tuple instruction with parameters"]
    #[format = "tuple {p1} {p2}"]
    TupleVariant(
        #[p1] i32,
        #[p2] u16,
        String, // non-parameter field
    ),

    #[description = "Simple tuple variant"]
    SimpleTuple(i32, String),
}

// Test all parameter positions p1-p5
#[allow(dead_code)]
#[derive(Debug, Clone, Opcode)]
pub enum AllParamsInstruction {
    #[description = "Instruction using all five parameters"]
    #[format = "full {p1} {p2} {p3} {p4} {p5}"]
    FullParams {
        #[p1]
        param1: i8,
        #[p2]
        param2: i16,
        #[p3]
        param3: i32,
        #[p4]
        param4: i64,
        #[p5]
        param5: u8,
        extra_field: String,
    },
}

// Test non-sequential parameter positions
#[allow(dead_code)]
#[derive(Debug, Clone, Opcode)]
pub enum NonSequentialParams {
    #[description = "Parameters not in order p1,p2,p3 but p1,p3,p5"]
    #[format = "sparse {p1} _ {p3} _ {p5}"]
    SparseParams {
        #[p1]
        first: i32,
        #[p3]
        third: i32,
        #[p5]
        fifth: i32,
        other: String,
    },
}

// Test complex format strings
#[derive(Debug, Clone, Opcode)]
pub enum ComplexFormatInstruction {
    #[description = "Complex format with special characters"]
    #[format = "complex: r[{p1}] := memory[{p2}+{p3}] (offset={p4})"]
    ComplexFormat {
        #[p1]
        register: usize,
        #[p2]
        base_addr: u32,
        #[p3]
        offset: i16,
        #[p4]
        extra_offset: i8,
    },
}

// Test various integer types
#[derive(Debug, Clone, Opcode)]
pub enum VariousTypesInstruction {
    #[description = "Test various integer types"]
    VariousTypes {
        #[p1]
        byte_val: u8,
        #[p2]
        word_val: u16,
        #[p3]
        dword_val: u32,
        #[p4]
        qword_val: u64,
        #[p5]
        signed_val: i64,
    },
}

// Test edge case: single character format
#[derive(Debug, Clone, Opcode)]
pub enum EdgeCaseInstruction {
    #[description = "Edge case with minimal format"]
    #[format = "{p1}"]
    Minimal {
        #[p1]
        value: i32,
    },

    #[description = "No format attribute - should use default"]
    NoFormat {
        #[p1]
        value: i32,
    },

    #[description = "Empty format string"]
    #[format = ""]
    EmptyFormat {
        #[p1]
        value: i32,
    },
}

// Test long descriptions and format strings
#[derive(Debug, Clone, Opcode)]
pub enum LongStringInstruction {
    #[description = "This is a very long description that spans multiple lines and contains lots of detail about what this instruction does, including complex explanations of edge cases and behavior"]
    #[format = "This is a very long format string with parameter {p1} and more text and parameter {p2} and even more descriptive text that goes on and on"]
    LongStrings {
        #[p1]
        param1: i32,
        #[p2]
        param2: i32,
    },
}

// Test special characters in descriptions and format
#[derive(Debug, Clone, Opcode)]
pub enum SpecialCharsInstruction {
    #[description = "Special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?`~"]
    #[format = "special: {p1} -> {p2} (cost=$100)"]
    SpecialChars {
        #[p1]
        input: i32,
        #[p2]
        output: i32,
    },
}

#[cfg(test)]
mod comprehensive_tests {
    use super::*;

    #[test]
    fn test_tuple_instruction() {
        let tuple_insn = TupleInstruction::TupleVariant(42, 100, "test".to_string());

        assert_eq!(tuple_insn.name(), "TupleVariant");
        assert_eq!(
            tuple_insn.description(),
            "Tuple instruction with parameters"
        );

        let explanation = tuple_insn.explain();
        assert_eq!(explanation.p1, 42);
        assert_eq!(explanation.p2, 100);
        assert_eq!(explanation.explanation, "tuple 42 100");
    }

    #[test]
    fn test_simple_tuple_default_format() {
        let simple = TupleInstruction::SimpleTuple(123, "test".to_string());

        let explanation = simple.explain();
        assert_eq!(explanation.name, "SimpleTuple");
        // Should use default format since no parameters marked
        assert_eq!(explanation.explanation, "SimpleTuple");
    }

    #[test]
    fn test_all_five_parameters() {
        let full = AllParamsInstruction::FullParams {
            param1: 1,
            param2: 2,
            param3: 3,
            param4: 4,
            param5: 5,
            extra_field: "extra".to_string(),
        };

        let explanation = full.explain();
        assert_eq!(explanation.p1, 1);
        assert_eq!(explanation.p2, 2);
        assert_eq!(explanation.p3, 3);
        assert_eq!(explanation.p4, 4);
        assert_eq!(explanation.p5, 5);
        assert_eq!(explanation.explanation, "full 1 2 3 4 5");
    }

    #[test]
    fn test_non_sequential_parameters() {
        let sparse = NonSequentialParams::SparseParams {
            first: 10,
            third: 30,
            fifth: 50,
            other: "other".to_string(),
        };

        let explanation = sparse.explain();
        assert_eq!(explanation.p1, 10);
        assert_eq!(explanation.p2, 0); // unused
        assert_eq!(explanation.p3, 30);
        assert_eq!(explanation.p4, 0); // unused
        assert_eq!(explanation.p5, 50);
        assert_eq!(explanation.explanation, "sparse 10 _ 30 _ 50");
    }

    #[test]
    fn test_complex_format_string() {
        let complex = ComplexFormatInstruction::ComplexFormat {
            register: 5,
            base_addr: 0x1000,
            offset: -10,
            extra_offset: 2,
        };

        let explanation = complex.explain();
        assert_eq!(explanation.p1, 5);
        assert_eq!(explanation.p2, 0x1000);
        assert_eq!(explanation.p3, -10);
        assert_eq!(explanation.p4, 2);
        assert_eq!(
            explanation.explanation,
            "complex: r[5] := memory[4096+-10] (offset=2)"
        );
    }

    #[test]
    fn test_various_integer_types() {
        let various = VariousTypesInstruction::VariousTypes {
            byte_val: 255,
            word_val: 65535,
            dword_val: 4294967295,
            qword_val: 18446744073709551615,
            signed_val: -9223372036854775808,
        };

        let explanation = various.explain();
        // All should be cast to i32
        assert_eq!(explanation.p1, 255i32);
        assert_eq!(explanation.p2, 65535i32);
        assert_eq!(explanation.p3, 4294967295u32 as i32); // will overflow/wrap
        assert_eq!(explanation.p4, 18446744073709551615u64 as i32); // will overflow/wrap
        assert_eq!(explanation.p5, -9223372036854775808i64 as i32); // will overflow/wrap
    }

    #[test]
    fn test_edge_case_formats() {
        let minimal = EdgeCaseInstruction::Minimal { value: 42 };
        let explanation = minimal.explain();
        assert_eq!(explanation.explanation, "42");

        let no_format = EdgeCaseInstruction::NoFormat { value: 42 };
        let explanation = no_format.explain();
        assert_eq!(explanation.explanation, "NoFormat 42"); // default format

        let empty_format = EdgeCaseInstruction::EmptyFormat { value: 42 };
        let explanation = empty_format.explain();
        assert_eq!(explanation.explanation, ""); // empty string
    }

    #[test]
    fn test_long_strings() {
        let long = LongStringInstruction::LongStrings {
            param1: 1,
            param2: 2,
        };

        assert!(long.description().len() > 100); // should be long

        let explanation = long.explain();
        assert!(explanation.explanation.contains("parameter 1"));
        assert!(explanation.explanation.contains("parameter 2"));
    }

    #[test]
    fn test_special_characters() {
        let special = SpecialCharsInstruction::SpecialChars {
            input: 10,
            output: 20,
        };

        assert!(special.description().contains("!@#$%^&*()"));

        let explanation = special.explain();
        assert_eq!(explanation.explanation, "special: 10 -> 20 (cost=$100)");
    }

    #[test]
    fn test_dictionary_generation() {
        let dict = TUPLEINSTRUCTION_OPCODE_DICTIONARY;
        assert_eq!(dict.len(), 2);

        let dict = ALLPARAMSINSTRUCTION_OPCODE_DICTIONARY;
        assert_eq!(dict.len(), 1);

        let dict = EDGECASEINSTRUCTION_OPCODE_DICTIONARY;
        assert_eq!(dict.len(), 3);
    }

    #[test]
    fn test_name_consistency() {
        // Ensure name() method returns same as what's in dictionary
        let minimal = EdgeCaseInstruction::Minimal { value: 42 };
        let dict = EDGECASEINSTRUCTION_OPCODE_DICTIONARY;

        let dict_entry = dict.iter().find(|d| d.name == "Minimal").unwrap();
        assert_eq!(minimal.name(), dict_entry.name);
        assert_eq!(minimal.description(), dict_entry.description);
    }

    #[test]
    fn test_zero_parameters() {
        let simple = TupleInstruction::SimpleTuple(123, "test".to_string());
        let explanation = simple.explain();

        // All parameters should be 0 when not used
        assert_eq!(explanation.p1, 0);
        assert_eq!(explanation.p2, 0);
        assert_eq!(explanation.p3, 0);
        assert_eq!(explanation.p4, 0);
        assert_eq!(explanation.p5, 0);
    }

    #[test]
    fn test_negative_values() {
        let sparse = NonSequentialParams::SparseParams {
            first: -100,
            third: -200,
            fifth: -300,
            other: "negative".to_string(),
        };

        let explanation = sparse.explain();
        assert_eq!(explanation.p1, -100);
        assert_eq!(explanation.p3, -200);
        assert_eq!(explanation.p5, -300);
        assert_eq!(explanation.explanation, "sparse -100 _ -200 _ -300");
    }
}

// Compilation tests - these should compile without errors
#[cfg(test)]
mod compilation_tests {
    use super::*;

    #[derive(Debug, Clone, Opcode)]
    #[allow(dead_code)]
    pub enum SimpleTestInstruction {
        #[description = "Simple test instruction"]
        Simple(i32),

        #[description = "Another test instruction"]
        Another,
    }

    const _TEST_NAME: &str = {
        match &SimpleTestInstruction::Simple(0) {
            SimpleTestInstruction::Simple(..) => "Simple",
            SimpleTestInstruction::Another => "Another",
        }
    };

    const _TEST_NAME_2: &str = {
        match &SimpleTestInstruction::Another {
            SimpleTestInstruction::Simple(..) => "Simple 2",
            SimpleTestInstruction::Another => "Another 2",
        }
    };

    const _TEST_DESCRIPTION: &str = {
        match &SimpleTestInstruction::Simple(0) {
            SimpleTestInstruction::Simple(..) => "Simple test instruction",
            SimpleTestInstruction::Another => "Another test instruction",
        }
    };

    #[test]
    fn test_const_functionality() {
        assert!(!_TEST_NAME.is_empty());
        assert!(!_TEST_NAME_2.is_empty());
        assert!(!_TEST_DESCRIPTION.is_empty());
    }
}
