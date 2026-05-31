; ModuleID = bmb_bootstrap
declare void @println(i64) nofree nounwind willreturn

define private noundef i64 @pi_times_100() norecurse alwaysinline mustprogress nounwind willreturn nosync "no-trapping-math"="true" uwtable memory(none) speculatable nofree {
entry:
  ret i64 314
}

define i64 @bmb_user_main() alwaysinline norecurse nosync nounwind nofree mustprogress "no-trapping-math"="true" uwtable {
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
; TBAA metadata
!900 = !{!"BMB TBAA"}
!901 = !{!"omnipotent char", !900, i64 0}
!902 = !{!"long long", !901, i64 0}
!903 = !{!902, !902, i64 0}
!904 = !{!"double", !901, i64 0}
!905 = !{!904, !904, i64 0}
!906 = !{!901, !901, i64 0}
; Branch weights: loop body likely (2000:1)
!907 = !{!"branch_weights", i32 2000, i32 1}

; v0.96.46: Inline main wrapper with runtime init
declare void @bmb_init_runtime(i32, ptr)
declare void @bmb_arena_destroy()
define noundef i32 @main(i32 %argc, ptr %argv) nounwind uwtable "no-trapping-math"="true" {
  call void @bmb_init_runtime(i32 %argc, ptr %argv)
  %r = call i64 @bmb_user_main()
  call void @bmb_arena_destroy()
  %rv = trunc i64 %r to i32
  ret i32 %rv
}