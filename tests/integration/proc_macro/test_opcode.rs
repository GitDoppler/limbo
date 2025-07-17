use turso_macros::Opcode;

#[derive(Debug, Clone, PartialEq)]
pub struct OpCodeDescription {
    pub name: &'static str,
    pub description: &'static str,
}

// Simple enum for basic testing with no parameters
#[derive(Debug, Clone, Opcode)]
#[allow(dead_code)]
pub enum SimpleInstruction {
    #[description = "No operation - does nothing"]
    Nop,

    #[description = "Halt execution"]
    Halt,
}

// Enum with single parameter to test basic parameter handling
#[derive(Debug, Clone, Opcode)]
pub enum BasicInstruction {
    #[description = "Jump to target address"]
    #[format = "goto {p1}"]
    Goto {
        #[p1]
        target: i32,
    },

    #[description = "Load immediate value"]
    LoadImm {
        #[p1]
        value: i32,
    },
}

// Test with multiple parameters
#[derive(Debug, Clone, Opcode)]
#[allow(dead_code)]
pub enum MultiParamInstruction {
    #[description = "Move value from register to register"]
    #[format = "r[{p2}] = r[{p1}]"]
    Move {
        #[p1]
        src: u8,
        #[p2]
        dst: u8,
        other_data: String,
    },

    #[description = "Add two values"]
    Add {
        #[p1]
        left: i16,
        #[p2]
        right: i16,
        #[p3]
        result: i16,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_no_params() {
        let nop = SimpleInstruction::Nop;
        assert_eq!(nop.name(), "Nop");
        assert_eq!(nop.description(), "No operation - does nothing");

        let explanation = nop.explain();
        assert_eq!(explanation.name, "Nop");
        assert_eq!(explanation.p1, 0);
        assert_eq!(explanation.p2, 0);
        assert_eq!(explanation.p3, 0);
        assert_eq!(explanation.p4, 0);
        assert_eq!(explanation.p5, 0);
    }

    #[test]
    fn test_basic_single_param() {
        let goto = BasicInstruction::Goto { target: 42 };
        assert_eq!(goto.name(), "Goto");
        assert_eq!(goto.description(), "Jump to target address");

        let explanation = goto.explain();
        assert_eq!(explanation.name, "Goto");
        assert_eq!(explanation.p1, 42);
        assert_eq!(explanation.explanation, "goto 42");
    }

    #[test]
    fn test_load_immediate() {
        let load = BasicInstruction::LoadImm { value: 123 };
        let explanation = load.explain();
        assert_eq!(explanation.name, "LoadImm");
        assert_eq!(explanation.p1, 123);
        // This should use default formatting since no format attribute
        assert_eq!(explanation.explanation, "LoadImm 123");
    }

    #[test]
    fn test_multi_param_with_format() {
        let mv = MultiParamInstruction::Move {
            src: 1,
            dst: 2,
            other_data: "test".to_string(),
        };

        let explanation = mv.explain();
        assert_eq!(explanation.name, "Move");
        assert_eq!(explanation.p1, 1);
        assert_eq!(explanation.p2, 2);
        assert_eq!(explanation.p3, 0); // unused
        assert_eq!(explanation.explanation, "r[2] = r[1]");
    }

    #[test]
    fn test_add_instruction() {
        let add = MultiParamInstruction::Add {
            left: 10,
            right: 20,
            result: 30,
        };

        let explanation = add.explain();
        assert_eq!(explanation.name, "Add");
        assert_eq!(explanation.p1, 10);
        assert_eq!(explanation.p2, 20);
        assert_eq!(explanation.p3, 30);
        // Should use default format with first two params
        assert_eq!(explanation.explanation, "Add 10 20");
    }

    #[test]
    fn test_opcode_dictionary() {
        let dict = SIMPLEINSTRUCTION_OPCODE_DICTIONARY;
        assert_eq!(dict.len(), 2);

        let nop_desc = dict.iter().find(|d| d.name == "Nop").unwrap();
        assert_eq!(nop_desc.description, "No operation - does nothing");

        let halt_desc = dict.iter().find(|d| d.name == "Halt").unwrap();
        assert_eq!(halt_desc.description, "Halt execution");
    }

    #[test]
    fn test_different_int_types() {
        // Test that different integer types are handled correctly
        let mv = MultiParamInstruction::Move {
            src: 255u8, // u8
            dst: 127u8, // u8
            other_data: "test".to_string(),
        };

        let explanation = mv.explain();
        assert_eq!(explanation.p1, 255i32);
        assert_eq!(explanation.p2, 127i32);
    }
}
