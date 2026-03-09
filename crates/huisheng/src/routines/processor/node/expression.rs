use std::collections::BTreeSet;

use egui_snarl::{NodeId, Snarl};
use evalexpr::{ContextWithMutableVariables, DefaultNumericTypes, HashMapContext};

use crate::{
    model::patch::node::{PatchNode, PatchNodeTrait, expression::Expression},
    node_or_bail,
    routines::processor::node::PatchNodeProcessable,
};

impl PatchNodeProcessable<'_> for Expression {
    type ProcessArg = ();
    fn process(
        node_id: NodeId,
        snarl: &mut Snarl<PatchNode>,
        _: Self::ProcessArg,
    ) -> Result<bool, ()> {
        let PatchNode::Expression(expr) = node_or_bail!(snarl, node_id) else {
            unreachable!();
        };

        let mut expr_str = None;
        if let Some(src) = expr.input_for_pin(Expression::INPUT_EXPR).to_owned() {
            expr_str = node_or_bail!(snarl, src.node).output_text(src.output);
        }

        let PatchNode::Expression(expr) = node_or_bail!(mut snarl, node_id) else {
            unreachable!();
        };
        if let Some(expr_str) = expr_str {
            expr.expr = expr_str;
        }

        if expr.expr != expr.last_expr {
            match evalexpr::build_operator_tree::<DefaultNumericTypes>(&expr.expr) {
                Ok(root) => {
                    expr.op_tree = Some(root);
                }
                Err(_) => {
                    expr.result = 0.0;
                    return Err(());
                }
            };

            expr.bindings = expr
                .op_tree
                .as_ref()
                .unwrap()
                .iter_variable_identifiers()
                .map(|s| (s.to_string(), 0.))
                .collect::<_>();
            expr.bindings.sort_by(|a, b| a.0.cmp(&b.0));

            expr.last_expr = expr.expr.clone();

            Expression::clear_all_connections_and_resize(node_id, snarl)?;
        }

        let PatchNode::Expression(expr) = node_or_bail!(snarl, node_id) else {
            unreachable!();
        };

        let mut input_ids_iter = expr.input_ids().into_iter();
        let _ = input_ids_iter.next();

        let mut values = Vec::new();
        for pin in input_ids_iter {
            if let Some(src) = pin.to_owned() {
                values.push(node_or_bail!(snarl, src.node).output_number(src.output));
            }
        }

        let PatchNode::Expression(expr) = node_or_bail!(mut snarl, node_id) else {
            unreachable!();
        };
        for (id, val) in values.iter().enumerate() {
            if let Some(val) = val {
                expr.bindings[id].1 = *val;
            }
        }

        let mut context: HashMapContext<DefaultNumericTypes> = HashMapContext::new();
        for (name, val) in &expr.bindings {
            context
                .set_value(name.clone(), evalexpr::Value::Float(*val))
                .unwrap();
        }
        expr.result = evalexpr::eval_float_with_context(&expr.expr, &context).unwrap_or(0.);

        Ok(false)
    }
}
