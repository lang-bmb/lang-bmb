; ModuleID = bmb_bootstrap
declare void @println(i64) nofree nounwind willreturn

define private noundef i64 @pi_times_100() norecurse alwaysinline mustprogress nounwind willreturn nosync "no-trapping-math"="true" uwtable memory(none) speculatable nofree {
entry:
  ret i64 314
}

define i64 @bmb_user_main() norecurse nofree mustprogress "no-trapping-math"="true" uwtable {
entry:
  %a_v0 = alloca i64
  store i64 42, ptr %a_v0
  %b_v1 = alloca i64
  store i64 42, ptr %b_v1
  call void @println(i64 42)
  %_t4 = load i64, ptr %b_v1
  call void @println(i64 %_t4)
  %_t6 = load i64, ptr %a_v0
  %_t7 = load i64, ptr %b_v1
  %_t8 = add nsw i64 %_t6, %_t7
  call void @println(i64 %_t8)
  call void @println(i64 0)
  %_t13 = call i64 @pi_times_100()
  %d_v13 = alloca i64
  store i64 %_t13, ptr %d_v13
  %e_v14 = alloca i64
  store i64 %_t13, ptr %e_v14
  call void @println(i64 %_t13)
  %_t17 = load i64, ptr %e_v14
  call void @println(i64 %_t17)
  %_t19 = load i64, ptr %d_v13
  %_t20 = load i64, ptr %e_v14
  %_t21 = add nsw i64 %_t19, %_t20
  call void @println(i64 %_t21)
  ret i64 0
}

!0 = !{}