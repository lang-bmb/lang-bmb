; ModuleID = bmb_bootstrap
declare void @println(i64) nofree nounwind willreturn

define private i64 @fib_loop(i64 noundef %n) norecurse inlinehint mustprogress nounwind willreturn nosync {
entry:
  %_t2_cmp = icmp sle i64 %n, 1
  br i1 %_t2_cmp, label %then_0, label %merge_0
then_0:
  ret i64 %n
merge_0:
  %a_7 = alloca i64
  store i64 0, ptr %a_7
  %b_8 = alloca i64
  store i64 1, ptr %b_8
  %_i_9 = alloca i64
  store i64 2, ptr %_i_9
  br label %for_start_1
for_start_1:
  %_t11 = load i64, ptr %_i_9
  %_t12_cmp = icmp slt i64 %_t11, %n
  br i1 %_t12_cmp, label %for_body_1, label %for_end_1
for_body_1:
  %_t13 = load i64, ptr %a_7
  %_t14 = load i64, ptr %b_8
  %_t15 = add nsw i64 %_t13, %_t14
  %_t16 = load i64, ptr %b_8
  store i64 %_t16, ptr %a_7
  store i64 %_t15, ptr %b_8
  br label %for_inc_1
for_inc_1:
  %_t20 = load i64, ptr %_i_9
  %_t22 = add nsw i64 %_t20, 1
  store i64 %_t22, ptr %_i_9
  br label %for_start_1
for_end_1:
  %_t24 = load i64, ptr %a_7
  %_t25 = load i64, ptr %b_8
  %_t26 = add nsw i64 %_t24, %_t25
  ret i64 %_t26
}

define private i64 @stair_climb(i64 noundef %n) norecurse inlinehint mustprogress nounwind willreturn nosync {
entry:
  %_t2_cmp = icmp sle i64 %n, 1
  br i1 %_t2_cmp, label %then_0, label %merge_0
then_0:
  ret i64 1
merge_0:
  %a_7 = alloca i64
  store i64 1, ptr %a_7
  %b_8 = alloca i64
  store i64 1, ptr %b_8
  %_i_9 = alloca i64
  store i64 2, ptr %_i_9
  br label %for_start_1
for_start_1:
  %_t11 = load i64, ptr %_i_9
  %_t12_cmp = icmp slt i64 %_t11, %n
  br i1 %_t12_cmp, label %for_body_1, label %for_end_1
for_body_1:
  %_t13 = load i64, ptr %a_7
  %_t14 = load i64, ptr %b_8
  %_t15 = add nsw i64 %_t13, %_t14
  %_t16 = load i64, ptr %b_8
  store i64 %_t16, ptr %a_7
  store i64 %_t15, ptr %b_8
  br label %for_inc_1
for_inc_1:
  %_t20 = load i64, ptr %_i_9
  %_t22 = add nsw i64 %_t20, 1
  store i64 %_t22, ptr %_i_9
  br label %for_start_1
for_end_1:
  %_t24 = load i64, ptr %a_7
  %_t25 = load i64, ptr %b_8
  %_t26 = add nsw i64 %_t24, %_t25
  ret i64 %_t26
}

define private i64 @kadane_5(i64 noundef %a, i64 noundef %b, i64 noundef %c, i64 noundef %d, i64 noundef %e) norecurse inlinehint mustprogress nounwind willreturn nosync {
entry:
  %max_ending_0 = alloca i64
  store i64 %a, ptr %max_ending_0
  %max_so_far_1 = alloca i64
  store i64 %a, ptr %max_so_far_1
  %_t5 = add nsw i64 %a, %b
  %_t6_cmp = icmp sgt i64 %b, %_t5
  br i1 %_t6_cmp, label %then_0, label %else_0
then_0:
  store i64 %b, ptr %max_ending_0
  br label %merge_0
else_0:
  %_t9 = load i64, ptr %max_ending_0
  %_t11 = add nsw i64 %_t9, %b
  store i64 %_t11, ptr %max_ending_0
  br label %merge_0
merge_0:
  %_t14 = load i64, ptr %max_ending_0
  %_t15 = load i64, ptr %max_so_far_1
  %_t16_cmp = icmp sgt i64 %_t14, %_t15
  br i1 %_t16_cmp, label %then_1, label %merge_1
then_1:
  %_t17 = load i64, ptr %max_ending_0
  store i64 %_t17, ptr %max_so_far_1
  br label %merge_1
merge_1:
  %_t22 = load i64, ptr %max_ending_0
  %_t24 = add nsw i64 %_t22, %c
  %_t25_cmp = icmp sgt i64 %c, %_t24
  br i1 %_t25_cmp, label %then_2, label %else_2
then_2:
  store i64 %c, ptr %max_ending_0
  br label %merge_2
else_2:
  %_t28 = load i64, ptr %max_ending_0
  %_t30 = add nsw i64 %_t28, %c
  store i64 %_t30, ptr %max_ending_0
  br label %merge_2
merge_2:
  %_t33 = load i64, ptr %max_ending_0
  %_t34 = load i64, ptr %max_so_far_1
  %_t35_cmp = icmp sgt i64 %_t33, %_t34
  br i1 %_t35_cmp, label %then_3, label %merge_3
then_3:
  %_t36 = load i64, ptr %max_ending_0
  store i64 %_t36, ptr %max_so_far_1
  br label %merge_3
merge_3:
  %_t41 = load i64, ptr %max_ending_0
  %_t43 = add nsw i64 %_t41, %d
  %_t44_cmp = icmp sgt i64 %d, %_t43
  br i1 %_t44_cmp, label %then_4, label %else_4
then_4:
  store i64 %d, ptr %max_ending_0
  br label %merge_4
else_4:
  %_t47 = load i64, ptr %max_ending_0
  %_t49 = add nsw i64 %_t47, %d
  store i64 %_t49, ptr %max_ending_0
  br label %merge_4
merge_4:
  %_t52 = load i64, ptr %max_ending_0
  %_t53 = load i64, ptr %max_so_far_1
  %_t54_cmp = icmp sgt i64 %_t52, %_t53
  br i1 %_t54_cmp, label %then_5, label %merge_5
then_5:
  %_t55 = load i64, ptr %max_ending_0
  store i64 %_t55, ptr %max_so_far_1
  br label %merge_5
merge_5:
  %_t60 = load i64, ptr %max_ending_0
  %_t62 = add nsw i64 %_t60, %e
  %_t63_cmp = icmp sgt i64 %e, %_t62
  br i1 %_t63_cmp, label %then_6, label %else_6
then_6:
  store i64 %e, ptr %max_ending_0
  br label %merge_6
else_6:
  %_t66 = load i64, ptr %max_ending_0
  %_t68 = add nsw i64 %_t66, %e
  store i64 %_t68, ptr %max_ending_0
  br label %merge_6
merge_6:
  %_t71 = load i64, ptr %max_ending_0
  %_t72 = load i64, ptr %max_so_far_1
  %_t73_cmp = icmp sgt i64 %_t71, %_t72
  br i1 %_t73_cmp, label %then_7, label %merge_7
then_7:
  %_t74 = load i64, ptr %max_ending_0
  store i64 %_t74, ptr %max_so_far_1
  br label %merge_7
merge_7:
  %_t78 = load i64, ptr %max_so_far_1
  ret i64 %_t78
}

define private i64 @tribonacci(i64 noundef %n) norecurse inlinehint mustprogress nounwind willreturn nosync {
entry:
  %_t2_cmp = icmp eq i64 %n, 0
  br i1 %_t2_cmp, label %then_0, label %merge_0
then_0:
  ret i64 0
merge_0:
  %_t9_cmp = icmp sle i64 %n, 2
  br i1 %_t9_cmp, label %then_1, label %merge_1
then_1:
  ret i64 1
merge_1:
  %a_14 = alloca i64
  store i64 0, ptr %a_14
  %b_15 = alloca i64
  store i64 1, ptr %b_15
  %c_16 = alloca i64
  store i64 1, ptr %c_16
  %_i_17 = alloca i64
  store i64 3, ptr %_i_17
  br label %for_start_2
for_start_2:
  %_t19 = load i64, ptr %_i_17
  %_t20_cmp = icmp slt i64 %_t19, %n
  br i1 %_t20_cmp, label %for_body_2, label %for_end_2
for_body_2:
  %_t21 = load i64, ptr %a_14
  %_t22 = load i64, ptr %b_15
  %_t23 = add nsw i64 %_t21, %_t22
  %_t24 = load i64, ptr %c_16
  %_t25 = add nsw i64 %_t23, %_t24
  %_t26 = load i64, ptr %b_15
  store i64 %_t26, ptr %a_14
  %_t28 = load i64, ptr %c_16
  store i64 %_t28, ptr %b_15
  store i64 %_t25, ptr %c_16
  br label %for_inc_2
for_inc_2:
  %_t32 = load i64, ptr %_i_17
  %_t34 = add nsw i64 %_t32, 1
  store i64 %_t34, ptr %_i_17
  br label %for_start_2
for_end_2:
  %_t36 = load i64, ptr %a_14
  %_t37 = load i64, ptr %b_15
  %_t38 = add nsw i64 %_t36, %_t37
  %_t39 = load i64, ptr %c_16
  %_t40 = add nsw i64 %_t38, %_t39
  ret i64 %_t40
}

define i64 @bmb_user_main() norecurse mustprogress {
entry:
  %_t1 = call i64 @fib_loop(i64 10)
  call void @println(i64 %_t1)
  %_t4 = call i64 @fib_loop(i64 20)
  call void @println(i64 %_t4)
  %_t7 = call i64 @stair_climb(i64 5)
  call void @println(i64 %_t7)
  %_t10 = call i64 @stair_climb(i64 10)
  call void @println(i64 %_t10)
  %_t23 = call i64 @kadane_5(i64 -2, i64 1, i64 -3, i64 4, i64 -1)
  call void @println(i64 %_t23)
  %_t32 = call i64 @kadane_5(i64 1, i64 2, i64 3, i64 -2, i64 5)
  call void @println(i64 %_t32)
  %_t35 = call i64 @tribonacci(i64 8)
  call void @println(i64 %_t35)
  %_t38 = call i64 @tribonacci(i64 10)
  call void @println(i64 %_t38)
  ret i64 0
}

!0 = !{}