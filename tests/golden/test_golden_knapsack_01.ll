; ModuleID = bmb_bootstrap
declare void @println(i64) nofree nounwind willreturn
declare noalias ptr @vec_new() nofree nosync nounwind willreturn
declare i64 @vec_push(ptr, i64) nofree nosync nounwind willreturn
declare i64 @vec_get(ptr, i64) nofree nosync nounwind willreturn memory(argmem: read)
declare void @vec_set(ptr, i64, i64) nofree nosync nounwind willreturn memory(argmem: readwrite)

define private i64 @dp_get(i64 noundef %dp, i64 noundef %cols, i64 noundef %r, i64 noundef %c) norecurse alwaysinline mustprogress nounwind willreturn nosync nofree memory(read) {
entry:
  %_t3 = mul nsw i64 %r, %cols
  %_t5 = add nsw i64 %_t3, %c
  %_t0_p0 = inttoptr i64 %dp to ptr
  %_t6 = call i64 @vec_get(ptr %_t0_p0, i64 %_t5)
  ret i64 %_t6
}

define private i64 @dp_set_val(i64 noundef %dp, i64 noundef %cols, i64 noundef %r, i64 noundef %c, i64 noundef %val) norecurse alwaysinline mustprogress nounwind willreturn nosync nofree memory(read) {
entry:
  %_t3 = mul nsw i64 %r, %cols
  %_t5 = add nsw i64 %_t3, %c
  %_t0_p0 = inttoptr i64 %dp to ptr
  call void @vec_set(ptr %_t0_p0, i64 %_t5, i64 %val)
  ret i64 0
}

define private i64 @max2(i64 noundef %a, i64 noundef %b) norecurse alwaysinline mustprogress nounwind willreturn nosync speculatable nofree memory(none) {
entry:
  %_t2_cmp = icmp sgt i64 %a, %b
  %_t5 = select i1 %_t2_cmp, i64 %a, i64 %b
  ret i64 %_t5
}

define private i64 @knapsack_fill_w(i64 noundef %dp, i64 noundef %wts, i64 noundef %vals, i64 noundef %n, i64 noundef %cap, i64 noundef %i, i64 noundef %w) norecurse inlinehint mustprogress nounwind willreturn nosync {
entry:
  br label %loop_header
loop_header:
  %dp_loop = phi i64 [ %dp, %entry ], [ %dp_loop, %merge_1 ]
  %wts_loop = phi i64 [ %wts, %entry ], [ %wts_loop, %merge_1 ]
  %vals_loop = phi i64 [ %vals, %entry ], [ %vals_loop, %merge_1 ]
  %n_loop = phi i64 [ %n, %entry ], [ %n_loop, %merge_1 ]
  %cap_loop = phi i64 [ %cap, %entry ], [ %cap_loop, %merge_1 ]
  %i_loop = phi i64 [ %i, %entry ], [ %i_loop, %merge_1 ]
  %w_loop = phi i64 [ %w, %entry ], [ %_t59, %merge_1 ]
  %_t2_cmp = icmp sgt i64 %w_loop, %cap_loop
  br i1 %_t2_cmp, label %then_0, label %else_0
then_0:
  br label %merge_0
else_0:
  %_t7 = sub nsw i64 %i_loop, 1
  %_t4_p0 = inttoptr i64 %wts_loop to ptr
  %_t8 = call i64 @vec_get(ptr %_t4_p0, i64 %_t7)
  %wi_v4 = alloca i64
  store i64 %_t8, ptr %wi_v4
  %_t12 = sub nsw i64 %i_loop, 1
  %_t9_p0 = inttoptr i64 %vals_loop to ptr
  %_t13 = call i64 @vec_get(ptr %_t9_p0, i64 %_t12)
  %vi_v9 = alloca i64
  store i64 %_t13, ptr %vi_v9
  %_t17 = add nsw i64 %cap_loop, 1
  %_t20 = sub nsw i64 %i_loop, 1
  %_t22 = call i64 @dp_get(i64 %dp_loop, i64 %_t17, i64 %_t20, i64 %w_loop)
  %without_v14 = alloca i64
  store i64 %_t22, ptr %without_v14
  %_t24 = load i64, ptr %wi_v4
  %_t25_cmp = icmp sge i64 %w_loop, %_t24
  br i1 %_t25_cmp, label %then_1, label %else_1
then_1:
  %_t26 = load i64, ptr %without_v14
  %_t30 = add nsw i64 %cap_loop, 1
  %_t33 = sub nsw i64 %i_loop, 1
  %_t35 = load i64, ptr %wi_v4
  %_t36 = sub nsw i64 %w_loop, %_t35
  %_t37 = call i64 @dp_get(i64 %dp_loop, i64 %_t30, i64 %_t33, i64 %_t36)
  %_t38 = load i64, ptr %vi_v9
  %_t39 = add nsw i64 %_t37, %_t38
  %_t40 = call i64 @max2(i64 %_t26, i64 %_t39)
  br label %merge_1
else_1:
  %_t41 = load i64, ptr %without_v14
  br label %merge_1
merge_1:
  %_t42 = phi i64 [ %_t40, %then_1 ], [ %_t41, %else_1 ]
  %_t46 = add nsw i64 %cap_loop, 1
  %_t50 = call i64 @dp_set_val(i64 %dp_loop, i64 %_t46, i64 %i_loop, i64 %w_loop, i64 %_t42)
  %_t59 = add nsw i64 %w_loop, 1
  br label %loop_header
merge_0:
  ret i64 0
}

define private i64 @knapsack_fill(i64 noundef %dp, i64 noundef %wts, i64 noundef %vals, i64 noundef %n, i64 noundef %cap, i64 noundef %i) norecurse inlinehint mustprogress nounwind willreturn nosync {
entry:
  br label %loop_header
loop_header:
  %dp_loop = phi i64 [ %dp, %entry ], [ %dp_loop, %else_0 ]
  %wts_loop = phi i64 [ %wts, %entry ], [ %wts_loop, %else_0 ]
  %vals_loop = phi i64 [ %vals, %entry ], [ %vals_loop, %else_0 ]
  %n_loop = phi i64 [ %n, %entry ], [ %n_loop, %else_0 ]
  %cap_loop = phi i64 [ %cap, %entry ], [ %cap_loop, %else_0 ]
  %i_loop = phi i64 [ %i, %entry ], [ %_t19, %else_0 ]
  %_t2_cmp = icmp sgt i64 %i_loop, %n_loop
  br i1 %_t2_cmp, label %then_0, label %else_0
then_0:
  br label %merge_0
else_0:
  %_t11 = call i64 @knapsack_fill_w(i64 %dp_loop, i64 %wts_loop, i64 %vals_loop, i64 %n_loop, i64 %cap_loop, i64 %i_loop, i64 0)
  %_t19 = add nsw i64 %i_loop, 1
  br label %loop_header
merge_0:
  ret i64 0
}

define private i64 @knapsack_trace(i64 noundef %dp, i64 noundef %wts, i64 noundef %sel, i64 noundef %cap, i64 noundef %i, i64 noundef %w) mustprogress nounwind willreturn nosync {
entry:
  %_t2_cmp = icmp sle i64 %i, 0
  br i1 %_t2_cmp, label %then_0, label %else_0
then_0:
  br label %merge_0
else_0:
  %_t7 = add nsw i64 %cap, 1
  %_t10 = call i64 @dp_get(i64 %dp, i64 %_t7, i64 %i, i64 %w)
  %cur_v4 = alloca i64
  store i64 %_t10, ptr %cur_v4
  %_t14 = add nsw i64 %cap, 1
  %_t17 = sub nsw i64 %i, 1
  %_t19 = call i64 @dp_get(i64 %dp, i64 %_t14, i64 %_t17, i64 %w)
  %_t20 = load i64, ptr %cur_v4
  %_t22_cmp = icmp ne i64 %_t20, %_t19
  br i1 %_t22_cmp, label %then_1, label %else_1
then_1:
  %_t26 = sub nsw i64 %i, 1
  %_t23_p0 = inttoptr i64 %sel to ptr
  call void @vec_set(ptr %_t23_p0, i64 %_t26, i64 1)
  %_t35 = sub nsw i64 %i, 1
  %_t40 = sub nsw i64 %i, 1
  %_t37_p0 = inttoptr i64 %wts to ptr
  %_t41 = call i64 @vec_get(ptr %_t37_p0, i64 %_t40)
  %_t42 = sub nsw i64 %w, %_t41
  %_t43 = tail call i64 @knapsack_trace(i64 %dp, i64 %wts, i64 %sel, i64 %cap, i64 %_t35, i64 %_t42)
  br label %merge_1
else_1:
  %_t50 = sub nsw i64 %i, 1
  %_t52 = tail call i64 @knapsack_trace(i64 %dp, i64 %wts, i64 %sel, i64 %cap, i64 %_t50, i64 %w)
  br label %merge_1
merge_1:
  %_t53 = phi i64 [ %_t43, %then_1 ], [ %_t52, %else_1 ]
  br label %merge_0
merge_0:
  %_t54 = phi i64 [ 0, %then_0 ], [ %_t53, %merge_1 ]
  ret i64 %_t54
}

define private i64 @count_sel(i64 noundef %sel, i64 noundef %n, i64 noundef %i, i64 noundef %acc) norecurse inlinehint mustprogress nounwind willreturn nosync nofree memory(read) {
entry:
  br label %loop_header
loop_header:
  %sel_loop = phi i64 [ %sel, %entry ], [ %sel_loop, %else_0 ]
  %n_loop = phi i64 [ %n, %entry ], [ %n_loop, %else_0 ]
  %i_loop = phi i64 [ %i, %entry ], [ %_t16, %else_0 ]
  %acc_loop = phi i64 [ %acc, %entry ], [ %_t19, %else_0 ]
  %_t2_cmp = icmp sge i64 %i_loop, %n_loop
  br i1 %_t2_cmp, label %merge_0, label %else_0
else_0:
  %_t4_p0 = inttoptr i64 %sel_loop to ptr
  %_t6 = call i64 @vec_get(ptr %_t4_p0, i64 %i_loop)
  %_t8_cmp = icmp eq i64 %_t6, 1
  %_t11 = select i1 %_t8_cmp, i64 1, i64 0
  %_t16 = add nsw i64 %i_loop, 1
  %_t19 = add nsw i64 %acc_loop, %_t11
  br label %loop_header
merge_0:
  ret i64 %acc_loop
}

define private i64 @weight_sel(i64 noundef %sel, i64 noundef %wts, i64 noundef %n, i64 noundef %i, i64 noundef %acc) norecurse inlinehint mustprogress nounwind willreturn nosync nofree memory(read) {
entry:
  br label %loop_header
loop_header:
  %sel_loop = phi i64 [ %sel, %entry ], [ %sel_loop, %merge_1 ]
  %wts_loop = phi i64 [ %wts, %entry ], [ %wts_loop, %merge_1 ]
  %n_loop = phi i64 [ %n, %entry ], [ %n_loop, %merge_1 ]
  %i_loop = phi i64 [ %i, %entry ], [ %_t19, %merge_1 ]
  %acc_loop = phi i64 [ %acc, %entry ], [ %_t22, %merge_1 ]
  %_t2_cmp = icmp sge i64 %i_loop, %n_loop
  br i1 %_t2_cmp, label %merge_0, label %else_0
else_0:
  %_t4_p0 = inttoptr i64 %sel_loop to ptr
  %_t6 = call i64 @vec_get(ptr %_t4_p0, i64 %i_loop)
  %_t8_cmp = icmp eq i64 %_t6, 1
  br i1 %_t8_cmp, label %then_1, label %else_1
then_1:
  %_t9_p0 = inttoptr i64 %wts_loop to ptr
  %_t11 = call i64 @vec_get(ptr %_t9_p0, i64 %i_loop)
  br label %merge_1
else_1:
  br label %merge_1
merge_1:
  %_t13 = phi i64 [ %_t11, %then_1 ], [ 0, %else_1 ]
  %_t19 = add nsw i64 %i_loop, 1
  %_t22 = add nsw i64 %acc_loop, %_t13
  br label %loop_header
merge_0:
  ret i64 %acc_loop
}

define private i64 @make_table(i64 noundef %sz) norecurse inlinehint mustprogress nounwind willreturn nosync {
entry:
  %_t0_ptr = call ptr @vec_new()
  %_t0 = ptrtoint ptr %_t0_ptr to i64
  %v_v0 = alloca i64
  store i64 %_t0, ptr %v_v0
  %_i_1 = alloca i64
  store i64 0, ptr %_i_1
  br label %for_start_0
for_start_0:
  %_t3 = load i64, ptr %_i_1
  %_t4_cmp = icmp slt i64 %_t3, %sz
  br i1 %_t4_cmp, label %for_body_0, label %for_end_0
for_body_0:
  %_t5 = load i64, ptr %v_v0
  %_t5_p0 = inttoptr i64 %_t5 to ptr
  %_t7 = call i64 @vec_push(ptr %_t5_p0, i64 0)
  br label %for_inc_0
for_inc_0:
  %_t8 = load i64, ptr %_i_1
  %_t10 = add nsw i64 %_t8, 1
  store i64 %_t10, ptr %_i_1
  br label %for_start_0
for_end_0:
  %_t12 = load i64, ptr %v_v0
  ret i64 %_t12
}

define private i64 @run_knapsack(i64 noundef %wts, i64 noundef %vals, i64 noundef %n, i64 noundef %cap) norecurse inlinehint mustprogress nounwind willreturn nosync {
entry:
  %_t2 = add nsw i64 %n, 1
  %_t5 = add nsw i64 %cap, 1
  %_t6 = mul nsw i64 %_t2, %_t5
  %_t7 = call i64 @make_table(i64 %_t6)
  %dp_v0 = alloca i64
  store i64 %_t7, ptr %dp_v0
  %_t14 = call i64 @knapsack_fill(i64 %_t7, i64 %wts, i64 %vals, i64 %n, i64 %cap, i64 1)
  %_t15 = load i64, ptr %dp_v0
  %_t18 = add nsw i64 %cap, 1
  %_t21 = call i64 @dp_get(i64 %_t15, i64 %_t18, i64 %n, i64 %cap)
  %opt_v15 = alloca i64
  store i64 %_t21, ptr %opt_v15
  call void @println(i64 %_t21)
  %_t25 = call i64 @make_table(i64 %n)
  %sel_v24 = alloca i64
  store i64 %_t25, ptr %sel_v24
  %_t26 = load i64, ptr %dp_v0
  %_t32 = call i64 @knapsack_trace(i64 %_t26, i64 %wts, i64 %_t25, i64 %cap, i64 %n, i64 %cap)
  %_t33 = load i64, ptr %sel_v24
  %_t37 = call i64 @count_sel(i64 %_t33, i64 %n, i64 0, i64 0)
  call void @println(i64 %_t37)
  %_t39 = load i64, ptr %sel_v24
  %_t44 = call i64 @weight_sel(i64 %_t39, i64 %wts, i64 %n, i64 0, i64 0)
  call void @println(i64 %_t44)
  %_t46 = load i64, ptr %opt_v15
  ret i64 %_t46
}

define i64 @bmb_user_main() norecurse mustprogress {
entry:
  %_t0_ptr = call ptr @vec_new()
  %_t0 = ptrtoint ptr %_t0_ptr to i64
  %w1_v0 = alloca i64
  store i64 %_t0, ptr %w1_v0
  %_t3 = call i64 @vec_push(ptr %_t0_ptr, i64 2)
  %_t4 = load i64, ptr %w1_v0
  %_t4_p0 = inttoptr i64 %_t4 to ptr
  %_t6 = call i64 @vec_push(ptr %_t4_p0, i64 3)
  %_t7 = load i64, ptr %w1_v0
  %_t7_p0 = inttoptr i64 %_t7 to ptr
  %_t9 = call i64 @vec_push(ptr %_t7_p0, i64 4)
  %_t10 = load i64, ptr %w1_v0
  %_t10_p0 = inttoptr i64 %_t10 to ptr
  %_t12 = call i64 @vec_push(ptr %_t10_p0, i64 5)
  %_t13_ptr = call ptr @vec_new()
  %_t13 = ptrtoint ptr %_t13_ptr to i64
  %v1_v13 = alloca i64
  store i64 %_t13, ptr %v1_v13
  %_t16 = call i64 @vec_push(ptr %_t13_ptr, i64 3)
  %_t17 = load i64, ptr %v1_v13
  %_t17_p0 = inttoptr i64 %_t17 to ptr
  %_t19 = call i64 @vec_push(ptr %_t17_p0, i64 4)
  %_t20 = load i64, ptr %v1_v13
  %_t20_p0 = inttoptr i64 %_t20 to ptr
  %_t22 = call i64 @vec_push(ptr %_t20_p0, i64 5)
  %_t23 = load i64, ptr %v1_v13
  %_t23_p0 = inttoptr i64 %_t23 to ptr
  %_t25 = call i64 @vec_push(ptr %_t23_p0, i64 6)
  %_t26 = load i64, ptr %w1_v0
  %_t27 = load i64, ptr %v1_v13
  %_t30 = call i64 @run_knapsack(i64 %_t26, i64 %_t27, i64 4, i64 8)
  %_t31_ptr = call ptr @vec_new()
  %_t31 = ptrtoint ptr %_t31_ptr to i64
  %w2_v31 = alloca i64
  store i64 %_t31, ptr %w2_v31
  %_t34 = call i64 @vec_push(ptr %_t31_ptr, i64 1)
  %_t35 = load i64, ptr %w2_v31
  %_t35_p0 = inttoptr i64 %_t35 to ptr
  %_t37 = call i64 @vec_push(ptr %_t35_p0, i64 1)
  %_t38 = load i64, ptr %w2_v31
  %_t38_p0 = inttoptr i64 %_t38 to ptr
  %_t40 = call i64 @vec_push(ptr %_t38_p0, i64 1)
  %_t41_ptr = call ptr @vec_new()
  %_t41 = ptrtoint ptr %_t41_ptr to i64
  %v2_v41 = alloca i64
  store i64 %_t41, ptr %v2_v41
  %_t44 = call i64 @vec_push(ptr %_t41_ptr, i64 10)
  %_t45 = load i64, ptr %v2_v41
  %_t45_p0 = inttoptr i64 %_t45 to ptr
  %_t47 = call i64 @vec_push(ptr %_t45_p0, i64 20)
  %_t48 = load i64, ptr %v2_v41
  %_t48_p0 = inttoptr i64 %_t48 to ptr
  %_t50 = call i64 @vec_push(ptr %_t48_p0, i64 30)
  %_t51 = load i64, ptr %w2_v31
  %_t52 = load i64, ptr %v2_v41
  %_t55 = call i64 @run_knapsack(i64 %_t51, i64 %_t52, i64 3, i64 5)
  %_t56_ptr = call ptr @vec_new()
  %_t56 = ptrtoint ptr %_t56_ptr to i64
  %w3_v56 = alloca i64
  store i64 %_t56, ptr %w3_v56
  %_t59 = call i64 @vec_push(ptr %_t56_ptr, i64 10)
  %_t60_ptr = call ptr @vec_new()
  %_t60 = ptrtoint ptr %_t60_ptr to i64
  %v3_v60 = alloca i64
  store i64 %_t60, ptr %v3_v60
  %_t63 = call i64 @vec_push(ptr %_t60_ptr, i64 100)
  %_t64 = load i64, ptr %w3_v56
  %_t65 = load i64, ptr %v3_v60
  %_t68 = call i64 @run_knapsack(i64 %_t64, i64 %_t65, i64 1, i64 5)
  %_t69_ptr = call ptr @vec_new()
  %_t69 = ptrtoint ptr %_t69_ptr to i64
  %w4_v69 = alloca i64
  store i64 %_t69, ptr %w4_v69
  %_t72 = call i64 @vec_push(ptr %_t69_ptr, i64 6)
  %_t73 = load i64, ptr %w4_v69
  %_t73_p0 = inttoptr i64 %_t73 to ptr
  %_t75 = call i64 @vec_push(ptr %_t73_p0, i64 5)
  %_t76 = load i64, ptr %w4_v69
  %_t76_p0 = inttoptr i64 %_t76 to ptr
  %_t78 = call i64 @vec_push(ptr %_t76_p0, i64 5)
  %_t79_ptr = call ptr @vec_new()
  %_t79 = ptrtoint ptr %_t79_ptr to i64
  %v4_v79 = alloca i64
  store i64 %_t79, ptr %v4_v79
  %_t82 = call i64 @vec_push(ptr %_t79_ptr, i64 6)
  %_t83 = load i64, ptr %v4_v79
  %_t83_p0 = inttoptr i64 %_t83 to ptr
  %_t85 = call i64 @vec_push(ptr %_t83_p0, i64 5)
  %_t86 = load i64, ptr %v4_v79
  %_t86_p0 = inttoptr i64 %_t86 to ptr
  %_t88 = call i64 @vec_push(ptr %_t86_p0, i64 5)
  %_t89 = load i64, ptr %w4_v69
  %_t90 = load i64, ptr %v4_v79
  %_t93 = call i64 @run_knapsack(i64 %_t89, i64 %_t90, i64 3, i64 10)
  %_t94_ptr = call ptr @vec_new()
  %_t94 = ptrtoint ptr %_t94_ptr to i64
  %w5_v94 = alloca i64
  store i64 %_t94, ptr %w5_v94
  %_t97 = call i64 @vec_push(ptr %_t94_ptr, i64 3)
  %_t98 = load i64, ptr %w5_v94
  %_t98_p0 = inttoptr i64 %_t98 to ptr
  %_t100 = call i64 @vec_push(ptr %_t98_p0, i64 4)
  %_t101 = load i64, ptr %w5_v94
  %_t101_p0 = inttoptr i64 %_t101 to ptr
  %_t103 = call i64 @vec_push(ptr %_t101_p0, i64 2)
  %_t104_ptr = call ptr @vec_new()
  %_t104 = ptrtoint ptr %_t104_ptr to i64
  %v5_v104 = alloca i64
  store i64 %_t104, ptr %v5_v104
  %_t107 = call i64 @vec_push(ptr %_t104_ptr, i64 4)
  %_t108 = load i64, ptr %v5_v104
  %_t108_p0 = inttoptr i64 %_t108 to ptr
  %_t110 = call i64 @vec_push(ptr %_t108_p0, i64 5)
  %_t111 = load i64, ptr %v5_v104
  %_t111_p0 = inttoptr i64 %_t111 to ptr
  %_t113 = call i64 @vec_push(ptr %_t111_p0, i64 3)
  %_t114 = load i64, ptr %w5_v94
  %_t115 = load i64, ptr %v5_v104
  %_t118 = call i64 @run_knapsack(i64 %_t114, i64 %_t115, i64 3, i64 7)
  %_t119_ptr = call ptr @vec_new()
  %_t119 = ptrtoint ptr %_t119_ptr to i64
  %w6_v119 = alloca i64
  store i64 %_t119, ptr %w6_v119
  %_t122 = call i64 @vec_push(ptr %_t119_ptr, i64 1)
  %_t123 = load i64, ptr %w6_v119
  %_t123_p0 = inttoptr i64 %_t123 to ptr
  %_t125 = call i64 @vec_push(ptr %_t123_p0, i64 2)
  %_t126 = load i64, ptr %w6_v119
  %_t126_p0 = inttoptr i64 %_t126 to ptr
  %_t128 = call i64 @vec_push(ptr %_t126_p0, i64 3)
  %_t129_ptr = call ptr @vec_new()
  %_t129 = ptrtoint ptr %_t129_ptr to i64
  %v6_v129 = alloca i64
  store i64 %_t129, ptr %v6_v129
  %_t132 = call i64 @vec_push(ptr %_t129_ptr, i64 6)
  %_t133 = load i64, ptr %v6_v129
  %_t133_p0 = inttoptr i64 %_t133 to ptr
  %_t135 = call i64 @vec_push(ptr %_t133_p0, i64 10)
  %_t136 = load i64, ptr %v6_v129
  %_t136_p0 = inttoptr i64 %_t136 to ptr
  %_t138 = call i64 @vec_push(ptr %_t136_p0, i64 12)
  %_t139 = load i64, ptr %w6_v119
  %_t140 = load i64, ptr %v6_v129
  %_t143 = call i64 @run_knapsack(i64 %_t139, i64 %_t140, i64 3, i64 5)
  ret i64 0
}

!0 = !{}