use std::fmt;
use std::collections::HashMap;
use itertools::Itertools;

struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    fn parse(input: &str) -> Self {
        peg::parser! {
            pub(crate) grammar parser() for str {
                pub(crate) rule root(p: &mut Program)
                    = (line(p) whitespace()*)* ![_]

                rule line(p: &mut Program)
                    = i:instruction() { p.instructions.push(i) }

                rule instruction() -> Instruction
                    = set_mask()
                    / assign()

                rule set_mask() -> Instruction
                    = "mask = " e:$(['X' | '0' | '1']+) {
                        let mut mask: Mask = Default::default();
                        for (i, x) in e.as_bytes().iter().rev().enumerate() {
                            match x {
                                b'1' => mask.set |= 2_u64.pow(i as _),
                                b'0' => mask.clear |= 2_u64.pow(i as _),
                                _ => {},
                            }
                        }
                        Instruction::SetMask(mask)
                    }

                rule assign() -> Instruction
                    = "mem[" addr:number() "] = " val:number() { Instruction::Assign { addr, val } }

                rule number() -> u64
                    = e:$(['0'..='9']+) { e.parse().unwrap() }

                rule whitespace()
                    = [' ' | '\t' | '\r' | '\n']
            }
        }

        let mut program = Program {
            instructions: Default::default(),
        };

        parser::root(input, &mut program).unwrap();
        program
    }
}

enum Instruction {
    SetMask(Mask),
    Assign { addr: u64, val: u64 },
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::SetMask(mask) => {
                write!(f, "mask: {:?}", mask)
            }
            Instruction::Assign { addr, val } => {
                write!(f, "mem[{}] = {}", addr, val)
            }
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Mask {
    set: u64,
    clear: u64,
}

impl fmt::Debug for Mask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //write!(f, "set {:036b}, clear {:036b}", self.set, self.clear)
        for i in 0..36 {
            let mask = i << (36 - i);
            write!(
                f,
                "{}",
                if self.set & mask != 0 {
                    '1'
                } else if self.clear & mask != 0 {
                    '0'
                } else {
                    'X'
                }
            )?;
        }
        Ok(())
    }
}

impl Mask {
    fn apply(&self, x: u64) -> u64 {
        (x | self.set) & (!self.clear)
    }

    fn or(&self, x: u64) -> Self {
        let mut res = *self;
        let set_or_clear = self.set | self.clear;

        for i in 0..36 {
            let mask = 1 << i;
            if set_or_clear & mask == 0 {
                // mask has X, it stays X.
            } else if x & mask != 0 {
                // x has 1, we set 1.
                res.set |= mask;
                res.clear &= !mask;
            } else {
                // otherwise, we leave whatever we had
            }
        }

        res
    }

    fn x_positions(&self) -> impl Iterator<Item = u64> + '_ {
        (0..36_u64).filter(move |i| ((1 << i) & (self.set | self.clear)) == 0)
    }

    fn apply_x(&self, positions: Vec<u64>) -> Self {
        // `Mask` is `Copy`, this is fine
        let mut res = *self;

        for pos in self.x_positions() {
            let mask = 1_u64 << pos;

            // In practice, `positions` is short, so it's not
            // worth constructing a `HashSet`
            if positions.contains(&pos) {
                // it's set, then!
                res.set |= mask;
            } else {
                // it not set, it's cleared
                res.clear |= mask;
            }
        }

        res
    }

    fn each_binary_value(&self) -> impl Iterator<Item = u64> + '_ {
        self.each_combination().map(|m| m.set)
    }

    fn each_combination(&self) -> impl Iterator<Item = Self> + '_ {
        self.x_positions()
            .powerset()
            .map(move |xes| self.apply_x(xes))
    }
}

fn main() {
    /*
    println!(
        "{:#?}",
        Program::parse(include_str!("input.txt")).instructions
    );
    */

    let mut mask: Mask = Default::default();
    let mut mem = HashMap::<u64, u64>::new();

    let program = Program::parse(include_str!("input.txt"));
    for ins in &program.instructions {
        match *ins {
            Instruction::SetMask(new_mask) => mask = new_mask,
            Instruction::Assign { addr, val } => {
                mem.insert(addr, mask.apply(val));
            }
        }
    }

    println!("Part 1:");
    println!("  Answer: {}", mem.values().sum::<u64>());

    let program = Program::parse(include_str!("input.txt"));
    /*
    println!("{:#?}", program.instructions);

    if let Instruction::SetMask(mask) = &program.instructions[0] {
        println!("{:?} (mask)", mask);
        let addr = 42;
        println!("{:036b} (addr)", addr);
        let mask = mask.or(addr);
        println!("{:?} (combined)", mask);

        println!("yields:");
        for val in mask.each_binary_value() {
            println!("{:036b} ({})", val, val);
        }
    }
    */

    let mut mask: Mask = Default::default();
    let mut mem = HashMap::<u64, u64>::new();

    for ins in &program.instructions {
        match *ins {
            Instruction::SetMask(new_mask) => mask = new_mask,
            Instruction::Assign { addr, val } => {
                for addr in mask.or(addr).each_binary_value() {
                    mem.insert(addr, val);
                }
            }
        }
    }

    println!("Part 2:");
    println!("  Answer: {}", mem.values().sum::<u64>());
}
