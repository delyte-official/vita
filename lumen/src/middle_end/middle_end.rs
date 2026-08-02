use crate::middle_end::*;
use crate::parser::BinaryOp;
use crate::type_checker::*;

pub fn lower(program: TypedProgram) -> IRProgram {
    let mut ir_functions = vec![];

    for func in program.functions {
        let mut next_temp = 0;
        let instructions = lower_stmts(func.body, &mut next_temp);
        ir_functions.push(IRFunction {
            name: func.name,
            instructions,
            local_count: func.local_count,
            temp_count: next_temp,
        });
    }

    IRProgram {
        functions: ir_functions,
    }
}

fn lower_stmts(stmts: Vec<TypedStmt>, next_temp: &mut usize) -> Vec<Instruction> {
    let mut instructions = vec![];
    for stmt in stmts {
        lower_stmt(stmt, &mut instructions, next_temp);
    }
    instructions
}

fn lower_stmt(stmt: TypedStmt, instructions: &mut Vec<Instruction>, next_temp: &mut usize) {
    match stmt {
        TypedStmt::Return(expr) => {
            let value = lower_expr(expr, instructions, next_temp);
            instructions.push(Instruction::Return(value));
        }
        TypedStmt::VarDecl(slot, expr) | TypedStmt::ValDecl(slot, expr) => {
            let value = lower_expr(expr, instructions, next_temp);
            instructions.push(Instruction::VarDecl(slot, value));
        }
        TypedStmt::If { condition, then_branch, elif_branches, else_branch } => {
            let cond_value = lower_expr(condition, instructions, next_temp);
            let then_instructions = lower_stmts(then_branch, next_temp);
            let else_instructions = lower_if_chain(elif_branches, else_branch, next_temp);
            instructions.push(Instruction::If {
                condition: cond_value,
                then_branch: then_instructions,
                else_branch: else_instructions,
            });
        }
        TypedStmt::Assign(slot, expr) => {
            let value = lower_expr(expr, instructions, next_temp);
            instructions.push(Instruction::Assign(slot, value));
        }
        TypedStmt::FuncCall(func_index) => {
            let dest = *next_temp;
            *next_temp += 1;
            instructions.push(Instruction::Call(dest, func_index));
        }
    }
}

fn lower_if_chain(
    elif_branches: Option<Vec<(TypedExpr, Vec<TypedStmt>)>>,
    else_branch: Option<Vec<TypedStmt>>,
    next_temp: &mut usize,
) -> Option<Vec<Instruction>> {
    let is_elif_empty = match &elif_branches {
        Some(branches) => branches.is_empty(),
        None => true,
    };
    if is_elif_empty {
        return else_branch.map(|stmts| lower_stmts(stmts, next_temp));
    }
    let mut branches = elif_branches.unwrap();
    let (condition, body) = branches.remove(0);

    let mut instructions = vec![];
    let cond_value = lower_expr(condition, &mut instructions, next_temp);
    let then_instructions = lower_stmts(body, next_temp);
    let rest = lower_if_chain(Some(branches), else_branch, next_temp);

    instructions.push(Instruction::If {
        condition: cond_value,
        then_branch: then_instructions,
        else_branch: rest,
    });
    Some(instructions)
}

fn lower_expr(expr: TypedExpr, instructions: &mut Vec<Instruction>, next_temp: &mut usize) -> Value {
    match expr {
        TypedExpr::Literal { value, ty } => match ty {
            ExprType::I32 => {
                let n: i64 = value
                    .parse()
                    .expect("literal was already validated as i32 by the type checker");
                Value::Const(n)
            }
            ExprType::Bool => Value::Const(if value == "true" { 1 } else { 0 }),
            ExprType::Char => Value::Const(
                value
                    .chars()
                    .next()
                    .expect("char literal was already validated to have exactly one character") as i64,
            ),
            ExprType::Str => {
                let dest = *next_temp;
                *next_temp += 1;
                instructions.push(Instruction::MakeString(dest, value));
                Value::Temp(dest)
            }
            ExprType::Struct(_) => unreachable!(
                "a plain Literal never carries a Struct type - struct values come from StructLiteral"
            ),
        },
        TypedExpr::StructLiteral { name, fields } => {
            let field_values: Vec<Value> = fields
                .into_iter()
                .map(|field| lower_expr(field, instructions, next_temp))
                .collect();
            let dest = *next_temp;
            *next_temp += 1;
            instructions.push(Instruction::MakeStruct { dest, name, fields: field_values });
            Value::Temp(dest)
        }
        TypedExpr::Template(parts) => lower_template(parts, instructions, next_temp),
        TypedExpr::Var(slot) => Value::Var(slot),
        TypedExpr::Binary { op, left, right, .. } => {
            let left_val = lower_expr(*left, instructions, next_temp);
            let right_val = lower_expr(*right, instructions, next_temp);
            let dest = *next_temp;
            *next_temp += 1;
            instructions.push(match op {
                BinaryOp::Plus => Instruction::Add(dest, left_val, right_val),
                BinaryOp::Minus => Instruction::Sub(dest, left_val, right_val),
                BinaryOp::Star => Instruction::Mul(dest, left_val, right_val),
                BinaryOp::Slash => Instruction::Div(dest, left_val, right_val),
            });
            Value::Temp(dest)
        }
        TypedExpr::FuncCall(func_index) => {
            let dest = *next_temp;
            *next_temp += 1;
            instructions.push(Instruction::Call(dest, func_index));
            Value::Temp(dest)
        }
    }
}

fn lower_template(parts: Vec<TypedTemplatePart>, instructions: &mut Vec<Instruction>, next_temp: &mut usize) -> Value {
    let mut acc: Option<Value> = None;

    for part in parts {
        let piece = match part {
            TypedTemplatePart::Text(s) => {
                let dest = *next_temp;
                *next_temp += 1;
                instructions.push(Instruction::MakeString(dest, s));
                Value::Temp(dest)
            }
            TypedTemplatePart::Expr(e) => {
                let ty = e.ty();
                let value = lower_expr(e, instructions, next_temp);
                match ty {
                    ExprType::Str => value,
                    ExprType::I32 => {
                        let dest = *next_temp;
                        *next_temp += 1;
                        instructions.push(Instruction::IntToString(dest, value));
                        Value::Temp(dest)
                    }
                    ExprType::Bool => {
                        let dest = *next_temp;
                        *next_temp += 1;
                        instructions.push(Instruction::BoolToString(dest, value));
                        Value::Temp(dest)
                    }
                    ExprType::Char => {
                        let dest = *next_temp;
                        *next_temp += 1;
                        instructions.push(Instruction::CharToString(dest, value));
                        Value::Temp(dest)
                    }
                    ExprType::Struct(_) => unreachable!(
                        "check_interpolatable already rejects struct values inside a template"
                    ),
                }
            }
        };

        acc = Some(match acc {
            None => piece,
            Some(prev) => {
                let dest = *next_temp;
                *next_temp += 1;
                instructions.push(Instruction::Concat(dest, prev, piece));
                Value::Temp(dest)
            }
        });
    }

    acc.unwrap_or_else(|| {
        let dest = *next_temp;
        *next_temp += 1;
        instructions.push(Instruction::MakeString(dest, String::new()));
        Value::Temp(dest)
    })
}
