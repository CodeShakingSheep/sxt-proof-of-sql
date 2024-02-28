use super::{prover_evaluate_equals_zero, prover_evaluate_or, BoolExpr, FilterExpr};
use crate::{
    base::{
        bit::BitDistribution,
        commitment::InnerProductProof,
        database::{
            make_random_test_accessor_data, ColumnType, OwnedTableTestAccessor,
            RandomTestAccessorDescriptor, RecordBatchTestAccessor, TestAccessor,
        },
        proof::{MessageLabel, TranscriptProtocol},
        scalar::ArkScalar,
    },
    owned_table, record_batch,
    sql::{
        ast::{
            test_expr::TestExprNode,
            test_utility::{col, cols_result, tab},
            BoolExprPlan,
        },
        proof::{
            make_transcript, Indexes, ProofBuilder, ProofExpr, QueryProof, ResultBuilder,
            VerifiableQueryResult,
        },
    },
};
use arrow::record_batch::RecordBatch;
use bumpalo::Bump;
use curve25519_dalek::RistrettoPoint;
use num_traits::Zero;
use polars::prelude::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;

#[test]
fn we_can_compare_a_constant_column() {
    let data = record_batch!(
        "a" => [123_i64, 123, 123],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 5.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
    let res = res
        .verify(&expr, &accessor, &())
        .unwrap()
        .into_record_batch();
    let expected = record_batch!(
        "b" => &[] as &[i64],
    );
    assert_eq!(res, expected);
}

#[test]
fn we_can_compare_a_varying_column_with_constant_sign() {
    let data = record_batch!(
        "a" => [123_i64, 567, 8],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 5.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
    let res = res
        .verify(&expr, &accessor, &())
        .unwrap()
        .into_record_batch();
    let expected = record_batch!(
        "b" => &[] as &[i64],
    );
    assert_eq!(res, expected);
}

#[test]
fn we_can_compare_a_varying_column_with_constant_absolute_value() {
    let data = record_batch!(
        "a" => [-123_i64, 123, -123],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 0.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
    let res = res
        .verify(&expr, &accessor, &())
        .unwrap()
        .into_record_batch();
    let expected = record_batch!(
        "b" => [1_i64, 3],
    );
    assert_eq!(res, expected);
}

#[test]
fn we_can_compare_a_constant_column_of_negative_columns() {
    let data = record_batch!(
        "a" => [-123_i64, -123, -123],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 5.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
    let res = res
        .verify(&expr, &accessor, &())
        .unwrap()
        .into_record_batch();
    let expected = record_batch!(
        "b" => [1_i64, 2, 3],
    );
    assert_eq!(res, expected);
}

#[test]
fn we_can_compare_a_varying_column_with_negative_only_signs() {
    let data = record_batch!(
        "a" => [-123_i64, -133, -823],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 5.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
    let res = res
        .verify(&expr, &accessor, &())
        .unwrap()
        .into_record_batch();
    let expected = record_batch!(
        "b" => [1_i64, 2, 3],
    );
    assert_eq!(res, expected);
}

#[test]
fn we_can_compare_a_column_with_varying_absolute_values_and_signs() {
    let data = record_batch!(
        "a" => [-1_i64, 9, 0],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 1.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
    let res = res
        .verify(&expr, &accessor, &())
        .unwrap()
        .into_record_batch();
    let expected = record_batch!(
        "b" => [1_i64, 3],
    );
    assert_eq!(res, expected);
}

#[test]
fn we_can_compare_column_with_greater_than_or_equal() {
    let data = record_batch!(
        "a" => [-1_i64, 9, 0],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 1.into(), false);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
    let res = res
        .verify(&expr, &accessor, &())
        .unwrap()
        .into_record_batch();
    let expected = record_batch!(
        "b" => [2_i64],
    );
    assert_eq!(res, expected);
}

#[test]
fn we_can_compare_a_column_with_varying_absolute_values_and_signs_and_a_constant_bit() {
    let data = record_batch!(
        "a" => [-2_i64, 3, 2],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 0.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
    let res = res
        .verify(&expr, &accessor, &())
        .unwrap()
        .into_record_batch();
    let expected = record_batch!(
        "b" => [1_i64],
    );
    assert_eq!(res, expected);
}

#[test]
fn we_can_compare_a_constant_column_of_zeros() {
    let data = record_batch!(
        "a" => [0_i64, 0, 0],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 0.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
    let res = res
        .verify(&expr, &accessor, &())
        .unwrap()
        .into_record_batch();
    let expected = record_batch!(
        "b" => [1_i64, 2, 3],
    );
    assert_eq!(res, expected);
}

#[test]
fn the_sign_can_be_0_or_1_for_a_constant_column_of_zeros() {
    let data = record_batch!(
        "a" => [0_i64, 0, 0],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 0.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let alloc = Bump::new();

    let mut result_builder = ResultBuilder::new(3);
    result_builder.set_result_indexes(Indexes::Sparse(vec![0, 1, 2]));
    let result_cols = cols_result(t, &["b"], &accessor);
    result_cols[0].result_evaluate(&mut result_builder, &accessor);

    let provable_result = result_builder.make_provable_query_result();
    let table_length = expr.get_length(&accessor);
    let generator_offset = expr.get_offset(&accessor);

    let mut transcript = make_transcript(&expr, &provable_result, table_length, generator_offset);
    transcript.challenge_ark_scalars(&mut [], MessageLabel::PostResultChallenges);

    let mut builder = ProofBuilder::new(3, 2, Vec::new());

    let lhs = [ArkScalar::zero(); 3];
    let equals_zero = prover_evaluate_equals_zero(&mut builder, &alloc, &lhs);

    let mut bit_distribution = BitDistribution {
        or_all: [0; 4],
        vary_mask: [0; 4],
    };
    bit_distribution.or_all[3] = 1 << 63;
    assert!(bit_distribution.sign_bit());
    builder.produce_bit_distribution(bit_distribution);
    let sign = [true; 3];
    prover_evaluate_or(&mut builder, &alloc, equals_zero, &sign);

    let selection = [true; 3];
    result_cols[0].prover_evaluate(&mut builder, &alloc, &accessor, &selection);

    let proof = QueryProof::<InnerProductProof>::new_from_builder(builder, 0, transcript, &());
    let res = proof
        .verify(&expr, &accessor, &provable_result, &())
        .unwrap()
        .into_record_batch();
    let expected = record_batch!(
        "b" => [1_i64, 2, 3],
    );
    assert_eq!(res, expected);
}

#[test]
fn verification_fails_if_commitments_dont_match_for_a_constant_column() {
    let data = record_batch!(
        "a" => [123_i64, 123, 123],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 5.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);

    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());

    let data = record_batch!(
        "a" => [321_i64, 321, 321],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 5.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    assert!(res.verify(&expr, &accessor, &()).is_err());
}

#[test]
fn verification_fails_if_commitments_dont_match_for_a_constant_absolute_column() {
    let data = record_batch!(
        "a" => [-123_i64, 123, -123],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 0.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());

    let data = record_batch!(
        "a" => [-321_i64, 321, -321],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 0.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    assert!(res.verify(&expr, &accessor, &()).is_err());
}

#[test]
fn verification_fails_if_commitments_dont_match_for_a_constant_sign_column() {
    let data = record_batch!(
        "a" => [193_i64, 323, 421],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 5.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());

    let data = record_batch!(
        "a" => [321_i64, 321, 321],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 5.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    assert!(res.verify(&expr, &accessor, &()).is_err());
}

#[test]
fn verification_fails_if_commitments_dont_match() {
    let data = record_batch!(
        "a" => [-523_i64, 923, 823],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 5.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());

    let data = record_batch!(
        "a" => [-523_i64, 923, 83],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = BoolExprPlan::new_inequality(col(t, "a", &accessor), 5.into(), true);
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    assert!(res.verify(&expr, &accessor, &()).is_err());
}

fn create_test_lte_expr<T: Into<ArkScalar> + Copy + Literal>(
    table_ref: &str,
    result_col: &str,
    filter_col: &str,
    filter_val: T,
    data: RecordBatch,
) -> TestExprNode {
    let mut accessor = RecordBatchTestAccessor::new_empty();
    let t = table_ref.parse().unwrap();
    accessor.add_table(t, data, 0);

    let where_clause =
        BoolExprPlan::new_inequality(col(t, filter_col, &accessor), filter_val.into(), true);

    let df_filter = polars::prelude::col(filter_col).lt_eq(lit(filter_val));
    TestExprNode::new(t, &[result_col], where_clause, df_filter, accessor)
}

#[test]
fn we_can_query_random_data_of_varying_size() {
    let descr = RandomTestAccessorDescriptor {
        min_rows: 1,
        max_rows: 20,
        min_value: -3,
        max_value: 3,
    };
    let mut rng = StdRng::from_seed([0u8; 32]);
    let cols = [("a", ColumnType::BigInt), ("b", ColumnType::BigInt)];
    for _ in 0..10 {
        let data = make_random_test_accessor_data(&mut rng, &cols, &descr);
        let filter_val = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let test_expr = create_test_lte_expr("sxt.t", "b", "a", filter_val, data);
        let res = test_expr.verify_expr();
        let expected = test_expr.query_table();
        assert_eq!(res, expected);
    }
}

#[test]
fn we_can_compute_the_correct_output_of_a_lte_inequality_expr_using_result_evaluate() {
    let data = owned_table!(
        "a" => [-1_i64, 9, 1],
        "b" => [1_i64, 2, 3],
    );
    let mut accessor = OwnedTableTestAccessor::new_empty();
    let t = "sxt.t".parse().unwrap();
    accessor.add_table(t, data, 0);
    let equals_expr =
        BoolExprPlan::<RistrettoPoint>::new_inequality(col(t, "a", &accessor), 1.into(), true);
    let alloc = Bump::new();
    let res = equals_expr.result_evaluate(3, &alloc, &accessor);
    let expected_res = &[true, false, true];
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compute_the_correct_output_of_a_gte_inequality_expr_using_result_evaluate() {
    let data = owned_table!(
        "a" => [-1_i64, 9, 1],
        "b" => [1_i64, 2, 3],
    );
    let mut accessor = OwnedTableTestAccessor::new_empty();
    let t = "sxt.t".parse().unwrap();
    accessor.add_table(t, data, 0);
    let equals_expr =
        BoolExprPlan::<RistrettoPoint>::new_inequality(col(t, "a", &accessor), 1.into(), false);
    let alloc = Bump::new();
    let res = equals_expr.result_evaluate(3, &alloc, &accessor);
    let expected_res = &[false, true, true];
    assert_eq!(res, expected_res);
}
