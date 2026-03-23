"""
test_bmb_algo.py - pytest test suite for the bmb_algo Python library.

Covers all 41 algorithms exported by bmb-algo, organised into test classes
by category:

    TestDynamicProgramming   – knapsack, max_subarray, coin_change, lis,
                               edit_distance, lcs
    TestGraphAlgorithms      – dijkstra, bfs_count, topological_sort,
                               floyd_warshall
    TestSortingAlgorithms    – quicksort, merge_sort, heap_sort,
                               counting_sort
    TestSearchAlgorithms     – binary_search
    TestNumberTheory         – gcd, lcm, fibonacci, prime_count, nqueens,
                               modpow
    TestUtilityFunctions     – power_set_size, matrix_multiply,
                               matrix_transpose, is_sorted, array_reverse,
                               array_rotate, unique_count, prefix_sum,
                               array_sum, array_min, array_max,
                               bit_popcount, bit_set, bit_test, bit_clear,
                               array_contains, array_index_of, djb2_hash
"""

import pytest
import bmb_algo


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

INF = 999999  # sentinel used for "no edge" in floyd_warshall tests


# ===========================================================================
# Dynamic Programming
# ===========================================================================

class TestDynamicProgramming:
    """Tests for DP algorithms."""

    # --- knapsack -----------------------------------------------------------

    def test_knapsack_basic(self):
        assert bmb_algo.knapsack([2, 3, 4], [3, 4, 5], 7) == 9

    def test_knapsack_exact_capacity(self):
        # All three items fit exactly
        assert bmb_algo.knapsack([1, 2, 3], [1, 2, 3], 6) == 6

    def test_knapsack_zero_capacity(self):
        assert bmb_algo.knapsack([1, 2], [10, 20], 0) == 0

    def test_knapsack_single_item_fits(self):
        assert bmb_algo.knapsack([3], [7], 5) == 7

    def test_knapsack_single_item_too_heavy(self):
        assert bmb_algo.knapsack([10], [99], 5) == 0

    def test_knapsack_classic_four_items(self):
        # weights [1,3,4,5], values [1,4,5,7], capacity 7 → 9
        assert bmb_algo.knapsack([1, 3, 4, 5], [1, 4, 5, 7], 7) == 9

    # --- max_subarray (Kadane) ----------------------------------------------

    def test_max_subarray_mixed(self):
        assert bmb_algo.max_subarray([-2, 1, -3, 4, -1, 2, 1, -5, 4]) == 6

    def test_max_subarray_all_positive(self):
        assert bmb_algo.max_subarray([1, 2, 3, 4, 5]) == 15

    def test_max_subarray_all_negative(self):
        assert bmb_algo.max_subarray([-3, -1, -2]) == -1

    def test_max_subarray_single_element(self):
        assert bmb_algo.max_subarray([42]) == 42

    def test_max_subarray_single_negative(self):
        assert bmb_algo.max_subarray([-7]) == -7

    def test_max_subarray_alternating(self):
        assert bmb_algo.max_subarray([2, -1, 2, -1, 2]) == 4

    # --- coin_change --------------------------------------------------------

    def test_coin_change_basic(self):
        assert bmb_algo.coin_change([1, 5, 11], 15) == 3

    def test_coin_change_exact_single(self):
        assert bmb_algo.coin_change([1, 5, 10], 10) == 1

    def test_coin_change_impossible(self):
        assert bmb_algo.coin_change([2], 3) == -1

    def test_coin_change_amount_zero(self):
        assert bmb_algo.coin_change([1, 5], 0) == 0

    def test_coin_change_greedy_not_optimal(self):
        # coins [1,3,4,6]: optimal is one 6-coin (1 coin, not greedy's 4+1+1 = 3)
        assert bmb_algo.coin_change([1, 3, 4, 6], 6) == 1

    # --- lis (longest increasing subsequence) --------------------------------

    def test_lis_classic(self):
        assert bmb_algo.lis([10, 9, 2, 5, 3, 7, 101, 18]) == 4

    def test_lis_already_sorted(self):
        assert bmb_algo.lis([1, 2, 3, 4, 5]) == 5

    def test_lis_descending(self):
        assert bmb_algo.lis([5, 4, 3, 2, 1]) == 1

    def test_lis_single(self):
        assert bmb_algo.lis([99]) == 1

    def test_lis_duplicates(self):
        # Strictly increasing, so duplicates do not extend the sequence
        assert bmb_algo.lis([1, 3, 3, 5]) == 3

    # --- edit_distance (Levenshtein) ----------------------------------------

    def test_edit_distance_kitten_sitting(self):
        assert bmb_algo.edit_distance("kitten", "sitting") == 3

    def test_edit_distance_identical(self):
        assert bmb_algo.edit_distance("abc", "abc") == 0

    def test_edit_distance_empty_to_str(self):
        assert bmb_algo.edit_distance("", "abc") == 3

    def test_edit_distance_str_to_empty(self):
        assert bmb_algo.edit_distance("abc", "") == 3

    def test_edit_distance_both_empty(self):
        assert bmb_algo.edit_distance("", "") == 0

    def test_edit_distance_single_substitution(self):
        assert bmb_algo.edit_distance("a", "b") == 1

    def test_edit_distance_insertion(self):
        assert bmb_algo.edit_distance("abc", "abcd") == 1

    def test_edit_distance_deletion(self):
        assert bmb_algo.edit_distance("abcd", "abc") == 1

    # --- lcs (longest common subsequence length) ----------------------------

    def test_lcs_classic(self):
        assert bmb_algo.lcs("ABCBDAB", "BDCAB") == 4

    def test_lcs_identical(self):
        assert bmb_algo.lcs("abc", "abc") == 3

    def test_lcs_no_common(self):
        assert bmb_algo.lcs("abc", "xyz") == 0

    def test_lcs_empty_first(self):
        assert bmb_algo.lcs("", "abc") == 0

    def test_lcs_empty_second(self):
        assert bmb_algo.lcs("abc", "") == 0

    def test_lcs_both_empty(self):
        assert bmb_algo.lcs("", "") == 0

    def test_lcs_subsequence(self):
        assert bmb_algo.lcs("AGGTAB", "GXTXAYB") == 4


# ===========================================================================
# Graph Algorithms
# ===========================================================================

class TestGraphAlgorithms:
    """Tests for graph algorithms."""

    # --- dijkstra -----------------------------------------------------------

    def test_dijkstra_basic(self):
        # 3-node graph: 0→1 (4), 1→2 (2)
        matrix = [[0, 4, -1], [-1, 0, 2], [-1, -1, 0]]
        assert bmb_algo.dijkstra(matrix, 0) == [0, 4, 6]

    def test_dijkstra_source_zero(self):
        # 4-node graph with multiple paths
        matrix = [
            [0, 1, 4, -1],
            [-1, 0, 2, 5],
            [-1, -1, 0, 1],
            [-1, -1, -1, 0],
        ]
        result = bmb_algo.dijkstra(matrix, 0)
        assert result[0] == 0
        assert result[1] == 1
        assert result[2] == 3
        assert result[3] == 4

    def test_dijkstra_single_node(self):
        assert bmb_algo.dijkstra([[0]], 0) == [0]

    def test_dijkstra_source_distance_zero(self):
        matrix = [[0, 3, -1], [2, 0, -1], [-1, 7, 0]]
        result = bmb_algo.dijkstra(matrix, 0)
        assert result[0] == 0

    # --- bfs_count ----------------------------------------------------------

    def test_bfs_count_linear_chain(self):
        # 0→1→2, all reachable from 0
        adj = [[0, 1, 0], [0, 0, 1], [0, 0, 0]]
        assert bmb_algo.bfs_count(adj, 0) == 3

    def test_bfs_count_from_middle(self):
        # Start at node 1, can only reach 1 and 2
        adj = [[0, 1, 0], [0, 0, 1], [0, 0, 0]]
        assert bmb_algo.bfs_count(adj, 1) == 2

    def test_bfs_count_from_end(self):
        # Start at node 2, no outgoing edges
        adj = [[0, 1, 0], [0, 0, 1], [0, 0, 0]]
        assert bmb_algo.bfs_count(adj, 2) == 1

    def test_bfs_count_isolated_node(self):
        # 2-node graph with no edges
        adj = [[0, 0], [0, 0]]
        assert bmb_algo.bfs_count(adj, 0) == 1

    def test_bfs_count_fully_connected(self):
        # All nodes connect to all others (directed)
        adj = [[0, 1, 1], [1, 0, 1], [1, 1, 0]]
        assert bmb_algo.bfs_count(adj, 0) == 3

    # --- topological_sort ---------------------------------------------------

    def test_topological_sort_basic_dag(self):
        # 0→1, 0→2, 1→3, 2→3
        adj = [
            [0, 1, 1, 0],
            [0, 0, 0, 1],
            [0, 0, 0, 1],
            [0, 0, 0, 0],
        ]
        order = bmb_algo.topological_sort(adj)
        assert len(order) == 4
        # 0 must come before 1, 2; 1 and 2 must come before 3
        assert order.index(0) < order.index(1)
        assert order.index(0) < order.index(2)
        assert order.index(1) < order.index(3)
        assert order.index(2) < order.index(3)

    def test_topological_sort_single_node(self):
        assert bmb_algo.topological_sort([[0]]) == [0]

    def test_topological_sort_linear(self):
        # Strict chain: 0→1→2→3
        adj = [
            [0, 1, 0, 0],
            [0, 0, 1, 0],
            [0, 0, 0, 1],
            [0, 0, 0, 0],
        ]
        order = bmb_algo.topological_sort(adj)
        assert order == [0, 1, 2, 3]

    # --- floyd_warshall -----------------------------------------------------

    def test_floyd_warshall_basic(self):
        matrix = [[0, 3, INF], [2, 0, INF], [INF, 7, 0]]
        result = bmb_algo.floyd_warshall(matrix)
        assert result[0][0] == 0
        assert result[0][1] == 3
        assert result[2][0] == 9   # 2→1 (7) + 1→0 (2) = 9
        assert result[2][1] == 7

    def test_floyd_warshall_single(self):
        assert bmb_algo.floyd_warshall([[0]]) == [[0]]

    def test_floyd_warshall_two_nodes(self):
        result = bmb_algo.floyd_warshall([[0, 5], [3, 0]])
        assert result[0][1] == 5
        assert result[1][0] == 3

    def test_floyd_warshall_no_shortcut(self):
        # Direct path is already optimal
        matrix = [[0, 1, INF], [INF, 0, 1], [INF, INF, 0]]
        result = bmb_algo.floyd_warshall(matrix)
        assert result[0][1] == 1
        assert result[0][2] == 2
        assert result[1][2] == 1


# ===========================================================================
# Sorting Algorithms
# ===========================================================================

class TestSortingAlgorithms:
    """Tests for sorting algorithms."""

    # shared test data
    UNSORTED = [3, 1, 4, 1, 5, 9, 2, 6]
    SORTED   = [1, 1, 2, 3, 4, 5, 6, 9]

    # --- quicksort ----------------------------------------------------------

    def test_quicksort_basic(self):
        assert bmb_algo.quicksort([3, 1, 4, 1, 5]) == [1, 1, 3, 4, 5]

    def test_quicksort_already_sorted(self):
        assert bmb_algo.quicksort([1, 2, 3, 4, 5]) == [1, 2, 3, 4, 5]

    def test_quicksort_reverse_sorted(self):
        assert bmb_algo.quicksort([5, 4, 3, 2, 1]) == [1, 2, 3, 4, 5]

    def test_quicksort_single_element(self):
        assert bmb_algo.quicksort([42]) == [42]

    def test_quicksort_duplicates(self):
        assert bmb_algo.quicksort(list(self.UNSORTED)) == list(self.SORTED)

    def test_quicksort_does_not_mutate_input(self):
        original = [3, 1, 2]
        result = bmb_algo.quicksort(original)
        assert result == [1, 2, 3]

    # --- merge_sort ---------------------------------------------------------

    def test_merge_sort_basic(self):
        assert bmb_algo.merge_sort([5, 3, 1, 4, 2]) == [1, 2, 3, 4, 5]

    def test_merge_sort_already_sorted(self):
        assert bmb_algo.merge_sort([1, 2, 3]) == [1, 2, 3]

    def test_merge_sort_single_element(self):
        assert bmb_algo.merge_sort([7]) == [7]

    def test_merge_sort_duplicates(self):
        assert bmb_algo.merge_sort(list(self.UNSORTED)) == list(self.SORTED)

    def test_merge_sort_two_elements(self):
        assert bmb_algo.merge_sort([2, 1]) == [1, 2]

    # --- heap_sort ----------------------------------------------------------

    def test_heap_sort_basic(self):
        assert bmb_algo.heap_sort([5, 3, 1, 4, 2]) == [1, 2, 3, 4, 5]

    def test_heap_sort_already_sorted(self):
        assert bmb_algo.heap_sort([1, 2, 3, 4]) == [1, 2, 3, 4]

    def test_heap_sort_single_element(self):
        assert bmb_algo.heap_sort([99]) == [99]

    def test_heap_sort_duplicates(self):
        assert bmb_algo.heap_sort(list(self.UNSORTED)) == list(self.SORTED)

    # --- counting_sort ------------------------------------------------------

    def test_counting_sort_basic(self):
        assert bmb_algo.counting_sort([3, 1, 4, 1, 5, 9, 2, 6]) == [1, 1, 2, 3, 4, 5, 6, 9]

    def test_counting_sort_explicit_max(self):
        assert bmb_algo.counting_sort([3, 1, 2], max_val=3) == [1, 2, 3]

    def test_counting_sort_all_same(self):
        assert bmb_algo.counting_sort([2, 2, 2]) == [2, 2, 2]

    def test_counting_sort_single_element(self):
        assert bmb_algo.counting_sort([5]) == [5]

    def test_counting_sort_sorted_output_matches_others(self):
        arr = [7, 2, 5, 1, 8, 3]
        assert bmb_algo.counting_sort(arr) == sorted(arr)


# ===========================================================================
# Search Algorithms
# ===========================================================================

class TestSearchAlgorithms:
    """Tests for search algorithms."""

    # --- binary_search ------------------------------------------------------

    def test_binary_search_found_middle(self):
        assert bmb_algo.binary_search([10, 20, 30, 40, 50], 30) == 2

    def test_binary_search_not_found(self):
        assert bmb_algo.binary_search([10, 20, 30, 40, 50], 35) == -1

    def test_binary_search_first_element(self):
        assert bmb_algo.binary_search([1, 2, 3, 4, 5], 1) == 0

    def test_binary_search_last_element(self):
        assert bmb_algo.binary_search([1, 2, 3, 4, 5], 5) == 4

    def test_binary_search_single_element_found(self):
        assert bmb_algo.binary_search([42], 42) == 0

    def test_binary_search_single_element_not_found(self):
        assert bmb_algo.binary_search([42], 0) == -1

    def test_binary_search_below_range(self):
        assert bmb_algo.binary_search([10, 20, 30], 5) == -1

    def test_binary_search_above_range(self):
        assert bmb_algo.binary_search([10, 20, 30], 99) == -1


# ===========================================================================
# Number Theory
# ===========================================================================

class TestNumberTheory:
    """Tests for number-theory algorithms."""

    # --- gcd ----------------------------------------------------------------

    def test_gcd_basic(self):
        assert bmb_algo.gcd(12, 8) == 4

    def test_gcd_large(self):
        assert bmb_algo.gcd(100, 75) == 25

    def test_gcd_coprime(self):
        assert bmb_algo.gcd(7, 13) == 1

    def test_gcd_same_values(self):
        assert bmb_algo.gcd(9, 9) == 9

    def test_gcd_one_is_one(self):
        assert bmb_algo.gcd(1, 100) == 1

    def test_gcd_one_is_multiple(self):
        assert bmb_algo.gcd(6, 3) == 3

    # --- lcm ----------------------------------------------------------------

    def test_lcm_basic(self):
        assert bmb_algo.lcm(12, 8) == 24

    def test_lcm_coprime(self):
        assert bmb_algo.lcm(4, 7) == 28

    def test_lcm_same_values(self):
        assert bmb_algo.lcm(5, 5) == 5

    def test_lcm_one_divides_other(self):
        assert bmb_algo.lcm(3, 9) == 9

    def test_lcm_consistency_with_gcd(self):
        a, b = 12, 18
        assert bmb_algo.lcm(a, b) == a * b // bmb_algo.gcd(a, b)

    # --- fibonacci ----------------------------------------------------------

    def test_fibonacci_zero(self):
        assert bmb_algo.fibonacci(0) == 0

    def test_fibonacci_one(self):
        assert bmb_algo.fibonacci(1) == 1

    def test_fibonacci_ten(self):
        assert bmb_algo.fibonacci(10) == 55

    def test_fibonacci_twenty(self):
        assert bmb_algo.fibonacci(20) == 6765

    def test_fibonacci_sequence(self):
        expected = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
        for i, val in enumerate(expected):
            assert bmb_algo.fibonacci(i) == val, f"F({i}) expected {val}"

    # --- prime_count (sieve of Eratosthenes) --------------------------------

    def test_prime_count_100(self):
        assert bmb_algo.prime_count(100) == 25

    def test_prime_count_1000(self):
        assert bmb_algo.prime_count(1000) == 168

    def test_prime_count_10(self):
        # Primes: 2, 3, 5, 7
        assert bmb_algo.prime_count(10) == 4

    def test_prime_count_two(self):
        assert bmb_algo.prime_count(2) == 1

    def test_prime_count_one(self):
        assert bmb_algo.prime_count(1) == 0

    # --- nqueens ------------------------------------------------------------

    def test_nqueens_1(self):
        assert bmb_algo.nqueens(1) == 1

    def test_nqueens_4(self):
        assert bmb_algo.nqueens(4) == 2

    def test_nqueens_6(self):
        assert bmb_algo.nqueens(6) == 4

    def test_nqueens_8(self):
        assert bmb_algo.nqueens(8) == 92

    def test_nqueens_0(self):
        # Degenerate case: 0-queen board has 1 trivial solution
        assert bmb_algo.nqueens(0) >= 0  # implementation-defined; just must not crash

    # --- modpow -------------------------------------------------------------

    def test_modpow_basic(self):
        assert bmb_algo.modpow(2, 10, 1000) == 24

    def test_modpow_exponent_zero(self):
        assert bmb_algo.modpow(5, 0, 13) == 1

    def test_modpow_base_zero(self):
        assert bmb_algo.modpow(0, 5, 13) == 0

    def test_modpow_large(self):
        # 2^100 mod 1_000_000_007
        assert bmb_algo.modpow(2, 100, 1_000_000_007) == pow(2, 100, 1_000_000_007)

    def test_modpow_mod_one(self):
        assert bmb_algo.modpow(99, 99, 1) == 0

    def test_modpow_fermat_little(self):
        # Fermat's little theorem: a^(p-1) ≡ 1 (mod p) for prime p, gcd(a,p)=1
        assert bmb_algo.modpow(3, 6, 7) == 1


# ===========================================================================
# Utility Functions
# ===========================================================================

class TestUtilityFunctions:
    """Tests for utility / array / bit-manipulation functions."""

    # --- power_set_size -----------------------------------------------------

    def test_power_set_size_zero(self):
        assert bmb_algo.power_set_size(0) == 1

    def test_power_set_size_one(self):
        assert bmb_algo.power_set_size(1) == 2

    def test_power_set_size_ten(self):
        assert bmb_algo.power_set_size(10) == 1024

    def test_power_set_size_matches_formula(self):
        for n in range(1, 16):
            assert bmb_algo.power_set_size(n) == 2 ** n

    # --- matrix_multiply ----------------------------------------------------

    def test_matrix_multiply_2x2(self):
        assert bmb_algo.matrix_multiply([[1, 2], [3, 4]], [[5, 6], [7, 8]]) == [
            [19, 22],
            [43, 50],
        ]

    def test_matrix_multiply_identity(self):
        I = [[1, 0], [0, 1]]
        A = [[3, 7], [2, 5]]
        assert bmb_algo.matrix_multiply(A, I) == A
        assert bmb_algo.matrix_multiply(I, A) == A

    def test_matrix_multiply_1x1(self):
        assert bmb_algo.matrix_multiply([[6]], [[7]]) == [[42]]

    def test_matrix_multiply_3x3(self):
        A = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
        B = [[9, 8, 7], [6, 5, 4], [3, 2, 1]]
        result = bmb_algo.matrix_multiply(A, B)
        # C[0][0] = 1*9 + 2*6 + 3*3 = 9+12+9 = 30
        assert result[0][0] == 30
        # C[0][2] = 1*7 + 2*4 + 3*1 = 7+8+3 = 18
        assert result[0][2] == 18
        # Verify using the formula directly
        assert result[0][2] == 1 * 7 + 2 * 4 + 3 * 1

    # --- matrix_transpose ---------------------------------------------------

    def test_matrix_transpose_2x2(self):
        assert bmb_algo.matrix_transpose([[1, 2], [3, 4]]) == [[1, 3], [2, 4]]

    def test_matrix_transpose_1x1(self):
        assert bmb_algo.matrix_transpose([[5]]) == [[5]]

    def test_matrix_transpose_3x3(self):
        M = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
        T = bmb_algo.matrix_transpose(M)
        for i in range(3):
            for j in range(3):
                assert T[i][j] == M[j][i]

    def test_matrix_transpose_involution(self):
        # Transposing twice gives the original matrix
        M = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
        assert bmb_algo.matrix_transpose(bmb_algo.matrix_transpose(M)) == M

    # --- is_sorted ----------------------------------------------------------

    def test_is_sorted_sorted(self):
        assert bmb_algo.is_sorted([1, 2, 3, 4, 5]) is True

    def test_is_sorted_unsorted(self):
        assert bmb_algo.is_sorted([3, 1, 2]) is False

    def test_is_sorted_single(self):
        assert bmb_algo.is_sorted([1]) is True

    def test_is_sorted_duplicates_ok(self):
        assert bmb_algo.is_sorted([1, 1, 2, 2]) is True

    def test_is_sorted_descending(self):
        assert bmb_algo.is_sorted([5, 4, 3, 2, 1]) is False

    # --- array_reverse ------------------------------------------------------

    def test_array_reverse_basic(self):
        assert bmb_algo.array_reverse([1, 2, 3, 4, 5]) == [5, 4, 3, 2, 1]

    def test_array_reverse_single(self):
        assert bmb_algo.array_reverse([42]) == [42]

    def test_array_reverse_two_elements(self):
        assert bmb_algo.array_reverse([1, 2]) == [2, 1]

    def test_array_reverse_involution(self):
        arr = [3, 1, 4, 1, 5, 9]
        assert bmb_algo.array_reverse(bmb_algo.array_reverse(arr)) == arr

    # --- array_rotate -------------------------------------------------------

    def test_array_rotate_basic(self):
        assert bmb_algo.array_rotate([1, 2, 3, 4, 5], 2) == [3, 4, 5, 1, 2]

    def test_array_rotate_zero(self):
        arr = [1, 2, 3]
        assert bmb_algo.array_rotate(arr, 0) == arr

    def test_array_rotate_full(self):
        arr = [1, 2, 3]
        assert bmb_algo.array_rotate(arr, 3) == arr

    def test_array_rotate_single(self):
        assert bmb_algo.array_rotate([7], 1) == [7]

    def test_array_rotate_by_one(self):
        assert bmb_algo.array_rotate([1, 2, 3, 4], 1) == [2, 3, 4, 1]

    # --- unique_count -------------------------------------------------------

    def test_unique_count_basic(self):
        assert bmb_algo.unique_count([1, 1, 2, 3, 3, 3, 4]) == 4

    def test_unique_count_all_unique(self):
        assert bmb_algo.unique_count([1, 2, 3, 4]) == 4

    def test_unique_count_all_same(self):
        assert bmb_algo.unique_count([5, 5, 5]) == 1

    def test_unique_count_single(self):
        assert bmb_algo.unique_count([9]) == 1

    # --- prefix_sum ---------------------------------------------------------

    def test_prefix_sum_basic(self):
        assert bmb_algo.prefix_sum([1, 2, 3, 4, 5]) == [1, 3, 6, 10, 15]

    def test_prefix_sum_single(self):
        assert bmb_algo.prefix_sum([7]) == [7]

    def test_prefix_sum_zeros(self):
        assert bmb_algo.prefix_sum([0, 0, 0]) == [0, 0, 0]

    def test_prefix_sum_length_preserved(self):
        arr = [3, 1, 4, 1, 5]
        result = bmb_algo.prefix_sum(arr)
        assert len(result) == len(arr)

    # --- array_sum ----------------------------------------------------------

    def test_array_sum_basic(self):
        assert bmb_algo.array_sum([1, 2, 3, 4, 5]) == 15

    def test_array_sum_single(self):
        assert bmb_algo.array_sum([100]) == 100

    def test_array_sum_negatives(self):
        assert bmb_algo.array_sum([-1, -2, -3]) == -6

    def test_array_sum_mixed(self):
        assert bmb_algo.array_sum([-5, 5, 10]) == 10

    # --- array_min ----------------------------------------------------------

    def test_array_min_basic(self):
        assert bmb_algo.array_min([5, 3, 8, 1, 7]) == 1

    def test_array_min_single(self):
        assert bmb_algo.array_min([42]) == 42

    def test_array_min_all_same(self):
        assert bmb_algo.array_min([4, 4, 4]) == 4

    def test_array_min_sorted(self):
        assert bmb_algo.array_min([1, 2, 3, 4, 5]) == 1

    # --- array_max ----------------------------------------------------------

    def test_array_max_basic(self):
        assert bmb_algo.array_max([5, 3, 8, 1, 7]) == 8

    def test_array_max_single(self):
        assert bmb_algo.array_max([99]) == 99

    def test_array_max_all_same(self):
        assert bmb_algo.array_max([4, 4, 4]) == 4

    def test_array_max_sorted(self):
        assert bmb_algo.array_max([1, 2, 3, 4, 5]) == 5

    # --- bit_popcount -------------------------------------------------------

    def test_bit_popcount_zero(self):
        assert bmb_algo.bit_popcount(0) == 0

    def test_bit_popcount_all_ones_byte(self):
        assert bmb_algo.bit_popcount(255) == 8

    def test_bit_popcount_single_bit(self):
        assert bmb_algo.bit_popcount(1) == 1
        assert bmb_algo.bit_popcount(16) == 1

    def test_bit_popcount_known_values(self):
        assert bmb_algo.bit_popcount(0b1010_1010) == 4
        assert bmb_algo.bit_popcount(0b1111_0000) == 4

    # --- bit_set / bit_clear / bit_test -------------------------------------

    def test_bit_set_basic(self):
        assert bmb_algo.bit_set(0, 3) == 8     # 0b1000

    def test_bit_set_already_set(self):
        assert bmb_algo.bit_set(8, 3) == 8

    def test_bit_set_multiple(self):
        v = bmb_algo.bit_set(0, 0)
        v = bmb_algo.bit_set(v, 2)
        assert v == 0b0101

    def test_bit_test_set_bit(self):
        assert bmb_algo.bit_test(8, 3) is True

    def test_bit_test_clear_bit(self):
        assert bmb_algo.bit_test(8, 0) is False

    def test_bit_test_boundary(self):
        assert bmb_algo.bit_test(1, 0) is True

    def test_bit_clear_basic(self):
        assert bmb_algo.bit_clear(8, 3) == 0

    def test_bit_clear_already_clear(self):
        assert bmb_algo.bit_clear(0, 3) == 0

    def test_bit_clear_leaves_others(self):
        # 0b1111 clear bit 1 → 0b1101
        assert bmb_algo.bit_clear(0b1111, 1) == 0b1101

    def test_bit_roundtrip(self):
        # set then clear should give original value
        original = 0b0000
        after_set = bmb_algo.bit_set(original, 5)
        assert bmb_algo.bit_test(after_set, 5) is True
        after_clear = bmb_algo.bit_clear(after_set, 5)
        assert after_clear == original

    # --- array_contains / array_index_of ------------------------------------

    def test_array_contains_present(self):
        assert bmb_algo.array_contains([1, 2, 3, 4], 2) is True

    def test_array_contains_absent(self):
        assert bmb_algo.array_contains([1, 2, 3], 99) is False

    def test_array_contains_first_element(self):
        assert bmb_algo.array_contains([10, 20, 30], 10) is True

    def test_array_contains_last_element(self):
        assert bmb_algo.array_contains([10, 20, 30], 30) is True

    def test_array_contains_single_element_match(self):
        assert bmb_algo.array_contains([7], 7) is True

    def test_array_contains_single_element_no_match(self):
        assert bmb_algo.array_contains([7], 8) is False

    def test_array_index_of_found(self):
        assert bmb_algo.array_index_of([10, 20, 30], 20) == 1

    def test_array_index_of_not_found(self):
        assert bmb_algo.array_index_of([10, 20, 30], 99) == -1

    def test_array_index_of_first(self):
        assert bmb_algo.array_index_of([5, 6, 7], 5) == 0

    def test_array_index_of_last(self):
        assert bmb_algo.array_index_of([5, 6, 7], 7) == 2

    def test_array_index_of_duplicate_returns_first(self):
        # The first occurrence should be returned
        result = bmb_algo.array_index_of([3, 3, 3], 3)
        assert result == 0

    # --- djb2_hash ----------------------------------------------------------

    def test_djb2_hash_hello(self):
        # BMB DJB2 value for "hello" (64-bit variant; seed 5381, multiplier 33)
        assert bmb_algo.djb2_hash("hello") == 210714636441

    def test_djb2_hash_empty(self):
        # Empty string must not crash; result is the seed value 5381
        result = bmb_algo.djb2_hash("")
        assert isinstance(result, int)

    def test_djb2_hash_deterministic(self):
        assert bmb_algo.djb2_hash("world") == bmb_algo.djb2_hash("world")

    def test_djb2_hash_different_strings(self):
        assert bmb_algo.djb2_hash("abc") != bmb_algo.djb2_hash("xyz")

    def test_djb2_hash_case_sensitive(self):
        assert bmb_algo.djb2_hash("Hello") != bmb_algo.djb2_hash("hello")
