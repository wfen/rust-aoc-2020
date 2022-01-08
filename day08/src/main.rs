use std::convert::TryInto;
use std::collections::HashSet;
use itertools::Itertools;

#[derive(Debug, Clone, Copy)]
enum InstructionKind {
    Nop,
    Acc,
    Jmp,
}

#[derive(Debug, Clone, Copy)]
struct Instruction {
    kind: InstructionKind,
    operand: isize,
}

type Program = Vec<Instruction>;

#[derive(Debug, Clone, Copy, Default)]
struct State {
    /// Program counter (instruction pointer)
    pc: usize,
    /// Accumulator
    acc: isize,
}

impl State {
    fn next(self, program: &Program) -> Self {
        let ins = program[self.pc];
        match ins.kind {
            InstructionKind::Nop => Self {
                pc: self.pc + 1,
                ..self
            },
            InstructionKind::Acc => Self {
                pc: self.pc + 1,
                acc: self.acc + ins.operand,
            },
            InstructionKind::Jmp => Self {
                pc: (self.pc as isize + ins.operand).try_into().unwrap(),
                ..self
            },
        }
    }
    fn next_option(self, program: &Program) -> Option<Self> {
        if !(0..program.len()).contains(&self.pc) {
            return None;
        }

        let ins = program[self.pc];
        Some(match ins.kind {
            InstructionKind::Nop => Self {
                pc: self.pc + 1,
                ..self
            },
            InstructionKind::Acc => Self {
                pc: self.pc + 1,
                acc: self.acc + ins.operand,
            },
            InstructionKind::Jmp => Self {
                pc: (self.pc as isize + ins.operand).try_into().unwrap(),
                ..self
            },
        })
    }
}

// parse_program() implements a quick manual parser
fn parse_program(input: &str) -> Program {
    input
        .lines()
        .map(|l| {
            let mut tokens = l.split(' ');
            Instruction {
                kind: match tokens.next() {
                    Some(tok) => match tok {
                        "nop" => InstructionKind::Nop,
                        "acc" => InstructionKind::Acc,
                        "jmp" => InstructionKind::Jmp,
                        _ => panic!("unknown instruction kind {}", tok)
                    },
                    None => panic!("for line {}, expected instruction kind", l),
                },
                operand: match tokens.next() {
                    Some(tok) => tok.parse().unwrap(),
                    None => panic!("for line {}, expected operand", l),
                },
            }
        })
        .collect()
}

fn main() {
    let program = parse_program(include_str!("input.txt"));
    //dbg!(program);

    //let mut state: State = Default::default();
    //dbg!("initial state", state);

    /*
    for _ in 0..5 {
        println!("will execute {:?}", program[state.pc]);
        state = state.next(&program);
        dbg!(state);
    }
    */

    /*
    let iter = std::iter::from_fn(|| {
        state = state.next(&program);
        Some(state)
    });
    */
    let mut iter = itertools::iterate(State::default(), |s| s.next(&program));
    //dbg!(iter.take(5).collect::<Vec<_>>());

    // We need to determine when we run an instruction for the second time, so we maintain a hashset of
    // all the instructions' positions we have already executed. Whenever HashSet::insert returns false
    // (it did have this value present), we stop and return what's in the accumulator.
    let mut set: HashSet<usize> = Default::default();
    let answer = iter.find(|state| !set.insert(state.pc)).unwrap();

    println!("Part 1:");
    println!(
        "  Before executing {} a second time, the accumulator was {}",
        answer.pc, answer.acc
    );

    /*
    let num_jmp_and_nop = program
        .iter()
        .filter(|i| matches!(i.kind, InstructionKind::Jmp | InstructionKind::Nop))
        .count();
    dbg!(num_jmp_and_nop);
    */

    println!("Part 2:");
    // This line was run initially, giving variant 196 terminated!... then the following three lines were run
    //find_variant(&program);
    let mut program = parse_program(include_str!("input.txt"));
    flip_kind(&mut program[196].kind);
    dbg!(eval(&program));

}

// we've identified the statement and flipped it. We iterate over the program using
fn eval(program: &Program) -> Option<isize> {
    itertools::iterate(Some(State::default()), |state| {
        state.and_then(|state| state.next_option(program))
    })
        .while_some()
        .last()
        .map(|s| s.acc)
}

fn flip_kind(kind: &mut InstructionKind) {
    *kind = match *kind {
        InstructionKind::Jmp => InstructionKind::Nop,
        InstructionKind::Nop => InstructionKind::Jmp,
        x => x,
    };
}

fn find_variant(program: &Program) {
    // filter_map + map generates all possible programs, and the second map evaluates
    // each program by iterating over its state as we keep evaluating instructions.
    let mut variants: Vec<_> = program
        .iter()
        .enumerate()
        .filter_map(|(index, ins)| match ins.kind {
            InstructionKind::Jmp | InstructionKind::Nop => Some(index),
            _ => None,
        })
        .map(|i| {
            let mut variant = program.clone();
            flip_kind(&mut variant[i].kind);
            (i, variant)
        })
        .map(|(index, variant)| {
            itertools::iterate(Some(State::default()), move |state| {
                state
                    .unwrap_or_else(|| panic!("variant {} terminated!", index))
                    .next_option(&variant)
            })
        })
        .collect();

    loop {
        for v in &mut variants {
            v.next();
        }
    }
}
