; ModuleID = 'tests/golden/test_golden_knapsack_01.ll'
source_filename = "tests/golden/test_golden_knapsack_01.ll"

; Function Attrs: mustprogress nofree nounwind willreturn
declare void @println(i64) local_unnamed_addr #0

; Function Attrs: mustprogress nofree nosync nounwind willreturn
declare noalias ptr @vec_new() local_unnamed_addr #1

; Function Attrs: mustprogress nofree nosync nounwind willreturn
declare i64 @vec_push(ptr, i64) local_unnamed_addr #1

; Function Attrs: mustprogress nofree nosync nounwind willreturn memory(argmem: read)
declare i64 @vec_get(ptr, i64) local_unnamed_addr #2

; Function Attrs: mustprogress nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @vec_set(ptr, i64, i64) local_unnamed_addr #3

; Function Attrs: inlinehint mustprogress nofree norecurse nosync nounwind willreturn
define private fastcc void @run_knapsack(i64 noundef %wts, i64 noundef range(i64 1, 5) %n, i64 noundef range(i64 5, 11) %cap) unnamed_addr #4 {
entry:
  %_t2 = add nuw nsw i64 %n, 1
  %_t5 = add nuw nsw i64 %cap, 1
  %_t6 = mul nuw nsw i64 %_t5, %_t2
  %_t0_ptr.i = tail call ptr @vec_new()
  br label %for_body_0.i

for_body_0.i:                                     ; preds = %for_body_0.i, %entry
  %_i_1.03.i = phi i64 [ 0, %entry ], [ %_t10.i, %for_body_0.i ]
  %_t7.i = tail call i64 @vec_push(ptr %_t0_ptr.i, i64 0)
  %_t10.i = add nuw nsw i64 %_i_1.03.i, 1
  %_t4_cmp.i = icmp samesign ult i64 %_t10.i, %_t6
  br i1 %_t4_cmp.i, label %for_body_0.i, label %make_table.exit

make_table.exit:                                  ; preds = %for_body_0.i
  %_t3.i = mul nuw nsw i64 %_t5, %n
  %_t5.i = add nuw nsw i64 %_t3.i, %cap
  %_t6.i = tail call i64 @vec_get(ptr %_t0_ptr.i, i64 %_t5.i)
  tail call void @println(i64 %_t6.i)
  %_t0_ptr.i3 = tail call ptr @vec_new()
  br label %for_body_0.i4

for_body_0.i4:                                    ; preds = %for_body_0.i4, %make_table.exit
  %_i_1.03.i5 = phi i64 [ 0, %make_table.exit ], [ %_t10.i7, %for_body_0.i4 ]
  %_t7.i6 = tail call i64 @vec_push(ptr %_t0_ptr.i3, i64 0)
  %_t10.i7 = add nuw nsw i64 %_i_1.03.i5, 1
  %_t4_cmp.i8 = icmp samesign ult i64 %_t10.i7, %n
  br i1 %_t4_cmp.i8, label %for_body_0.i4, label %make_table.exit10

make_table.exit10:                                ; preds = %for_body_0.i4
  %_t37_p0.i = inttoptr i64 %wts to ptr
  br label %else_0.lr.ph.i

else_0.lr.ph.i:                                   ; preds = %then_1.i, %make_table.exit10
  %w.tr.ph17.i = phi i64 [ %cap, %make_table.exit10 ], [ %_t42.i, %then_1.i ]
  %i.tr.ph16.i = phi i64 [ %n, %make_table.exit10 ], [ %_t17.i, %then_1.i ]
  br label %else_0.i

tailrecurse.i:                                    ; preds = %else_0.i
  %_t2_cmp.i = icmp eq i64 %_t17.i, 0
  br i1 %_t2_cmp.i, label %else_0.i12.preheader, label %else_0.i

else_0.i:                                         ; preds = %tailrecurse.i, %else_0.lr.ph.i
  %i.tr10.i = phi i64 [ %i.tr.ph16.i, %else_0.lr.ph.i ], [ %_t17.i, %tailrecurse.i ]
  %_t3.i1.i = mul nuw nsw i64 %i.tr10.i, %_t5
  %_t5.i2.i = add nsw i64 %_t3.i1.i, %w.tr.ph17.i
  %_t6.i4.i = tail call i64 @vec_get(ptr %_t0_ptr.i, i64 %_t5.i2.i)
  %_t17.i = add nsw i64 %i.tr10.i, -1
  %_t3.i.i = mul nuw nsw i64 %_t17.i, %_t5
  %_t5.i.i = add nsw i64 %_t3.i.i, %w.tr.ph17.i
  %_t6.i.i = tail call i64 @vec_get(ptr %_t0_ptr.i, i64 %_t5.i.i)
  %_t22_cmp.not.i = icmp eq i64 %_t6.i4.i, %_t6.i.i
  br i1 %_t22_cmp.not.i, label %tailrecurse.i, label %then_1.i

then_1.i:                                         ; preds = %else_0.i
  tail call void @vec_set(ptr %_t0_ptr.i3, i64 %_t17.i, i64 1)
  %_t41.i = tail call i64 @vec_get(ptr %_t37_p0.i, i64 %_t17.i)
  %_t42.i = sub nsw i64 %w.tr.ph17.i, %_t41.i
  %_t2_cmp9.i = icmp eq i64 %_t17.i, 0
  br i1 %_t2_cmp9.i, label %else_0.i12.preheader, label %else_0.lr.ph.i

else_0.i12.preheader:                             ; preds = %then_1.i, %tailrecurse.i
  br label %else_0.i12

else_0.i12:                                       ; preds = %else_0.i12.preheader, %else_0.i12
  %acc_loop2.i = phi i64 [ %_t19.i, %else_0.i12 ], [ 0, %else_0.i12.preheader ]
  %i_loop1.i = phi i64 [ %_t16.i, %else_0.i12 ], [ 0, %else_0.i12.preheader ]
  %_t6.i13 = tail call i64 @vec_get(ptr %_t0_ptr.i3, i64 %i_loop1.i)
  %_t8_cmp.i = icmp eq i64 %_t6.i13, 1
  %_t11.i = zext i1 %_t8_cmp.i to i64
  %_t16.i = add nuw nsw i64 %i_loop1.i, 1
  %_t19.i = add nuw nsw i64 %acc_loop2.i, %_t11.i
  %_t2_cmp.not.i = icmp samesign ult i64 %_t16.i, %n
  br i1 %_t2_cmp.not.i, label %else_0.i12, label %count_sel.exit

count_sel.exit:                                   ; preds = %else_0.i12
  tail call void @println(i64 %_t19.i)
  br label %else_0.i14

else_0.i14:                                       ; preds = %merge_1.i, %count_sel.exit
  %acc_loop2.i15 = phi i64 [ 0, %count_sel.exit ], [ %_t22.i, %merge_1.i ]
  %i_loop1.i16 = phi i64 [ 0, %count_sel.exit ], [ %_t19.i19, %merge_1.i ]
  %_t6.i17 = tail call i64 @vec_get(ptr %_t0_ptr.i3, i64 %i_loop1.i16)
  %_t8_cmp.i18 = icmp eq i64 %_t6.i17, 1
  br i1 %_t8_cmp.i18, label %then_1.i21, label %merge_1.i

then_1.i21:                                       ; preds = %else_0.i14
  %_t11.i22 = tail call i64 @vec_get(ptr %_t37_p0.i, i64 %i_loop1.i16)
  br label %merge_1.i

merge_1.i:                                        ; preds = %then_1.i21, %else_0.i14
  %_t13.i = phi i64 [ %_t11.i22, %then_1.i21 ], [ 0, %else_0.i14 ]
  %_t19.i19 = add nuw nsw i64 %i_loop1.i16, 1
  %_t22.i = add nsw i64 %_t13.i, %acc_loop2.i15
  %_t2_cmp.not.i20 = icmp samesign ult i64 %_t19.i19, %n
  br i1 %_t2_cmp.not.i20, label %else_0.i14, label %weight_sel.exit

weight_sel.exit:                                  ; preds = %merge_1.i
  tail call void @println(i64 %_t22.i)
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn
define noundef i64 @bmb_user_main() local_unnamed_addr #5 {
entry:
  %_t0_ptr = tail call ptr @vec_new()
  %_t0 = ptrtoint ptr %_t0_ptr to i64
  %_t3 = tail call i64 @vec_push(ptr %_t0_ptr, i64 2)
  %_t6 = tail call i64 @vec_push(ptr %_t0_ptr, i64 3)
  %_t9 = tail call i64 @vec_push(ptr %_t0_ptr, i64 4)
  %_t12 = tail call i64 @vec_push(ptr %_t0_ptr, i64 5)
  %_t13_ptr = tail call ptr @vec_new()
  %_t16 = tail call i64 @vec_push(ptr %_t13_ptr, i64 3)
  %_t19 = tail call i64 @vec_push(ptr %_t13_ptr, i64 4)
  %_t22 = tail call i64 @vec_push(ptr %_t13_ptr, i64 5)
  %_t25 = tail call i64 @vec_push(ptr %_t13_ptr, i64 6)
  tail call fastcc void @run_knapsack(i64 %_t0, i64 4, i64 8)
  %_t31_ptr = tail call ptr @vec_new()
  %_t31 = ptrtoint ptr %_t31_ptr to i64
  %_t34 = tail call i64 @vec_push(ptr %_t31_ptr, i64 1)
  %_t37 = tail call i64 @vec_push(ptr %_t31_ptr, i64 1)
  %_t40 = tail call i64 @vec_push(ptr %_t31_ptr, i64 1)
  %_t41_ptr = tail call ptr @vec_new()
  %_t44 = tail call i64 @vec_push(ptr %_t41_ptr, i64 10)
  %_t47 = tail call i64 @vec_push(ptr %_t41_ptr, i64 20)
  %_t50 = tail call i64 @vec_push(ptr %_t41_ptr, i64 30)
  tail call fastcc void @run_knapsack(i64 %_t31, i64 3, i64 5)
  %_t56_ptr = tail call ptr @vec_new()
  %_t56 = ptrtoint ptr %_t56_ptr to i64
  %_t59 = tail call i64 @vec_push(ptr %_t56_ptr, i64 10)
  %_t60_ptr = tail call ptr @vec_new()
  %_t63 = tail call i64 @vec_push(ptr %_t60_ptr, i64 100)
  tail call fastcc void @run_knapsack(i64 %_t56, i64 1, i64 5)
  %_t69_ptr = tail call ptr @vec_new()
  %_t69 = ptrtoint ptr %_t69_ptr to i64
  %_t72 = tail call i64 @vec_push(ptr %_t69_ptr, i64 6)
  %_t75 = tail call i64 @vec_push(ptr %_t69_ptr, i64 5)
  %_t78 = tail call i64 @vec_push(ptr %_t69_ptr, i64 5)
  %_t79_ptr = tail call ptr @vec_new()
  %_t82 = tail call i64 @vec_push(ptr %_t79_ptr, i64 6)
  %_t85 = tail call i64 @vec_push(ptr %_t79_ptr, i64 5)
  %_t88 = tail call i64 @vec_push(ptr %_t79_ptr, i64 5)
  tail call fastcc void @run_knapsack(i64 %_t69, i64 3, i64 10)
  %_t94_ptr = tail call ptr @vec_new()
  %_t94 = ptrtoint ptr %_t94_ptr to i64
  %_t97 = tail call i64 @vec_push(ptr %_t94_ptr, i64 3)
  %_t100 = tail call i64 @vec_push(ptr %_t94_ptr, i64 4)
  %_t103 = tail call i64 @vec_push(ptr %_t94_ptr, i64 2)
  %_t104_ptr = tail call ptr @vec_new()
  %_t107 = tail call i64 @vec_push(ptr %_t104_ptr, i64 4)
  %_t110 = tail call i64 @vec_push(ptr %_t104_ptr, i64 5)
  %_t113 = tail call i64 @vec_push(ptr %_t104_ptr, i64 3)
  tail call fastcc void @run_knapsack(i64 %_t94, i64 3, i64 7)
  %_t119_ptr = tail call ptr @vec_new()
  %_t119 = ptrtoint ptr %_t119_ptr to i64
  %_t122 = tail call i64 @vec_push(ptr %_t119_ptr, i64 1)
  %_t125 = tail call i64 @vec_push(ptr %_t119_ptr, i64 2)
  %_t128 = tail call i64 @vec_push(ptr %_t119_ptr, i64 3)
  %_t129_ptr = tail call ptr @vec_new()
  %_t132 = tail call i64 @vec_push(ptr %_t129_ptr, i64 6)
  %_t135 = tail call i64 @vec_push(ptr %_t129_ptr, i64 10)
  %_t138 = tail call i64 @vec_push(ptr %_t129_ptr, i64 12)
  tail call fastcc void @run_knapsack(i64 %_t119, i64 3, i64 5)
  ret i64 0
}

attributes #0 = { mustprogress nofree nounwind willreturn }
attributes #1 = { mustprogress nofree nosync nounwind willreturn }
attributes #2 = { mustprogress nofree nosync nounwind willreturn memory(argmem: read) }
attributes #3 = { mustprogress nofree nosync nounwind willreturn memory(argmem: readwrite) }
attributes #4 = { inlinehint mustprogress nofree norecurse nosync nounwind willreturn }
attributes #5 = { mustprogress nofree norecurse nosync nounwind willreturn }
