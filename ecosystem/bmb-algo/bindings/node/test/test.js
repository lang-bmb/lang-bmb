'use strict';
const { test } = require('node:test');
const assert = require('node:assert/strict');
const algo = require('..');

// koffi returns regular JS numbers for int64_t (BigInt only for values > 2^53)

// Number theory
test('gcd', () => {
  assert.equal(algo.gcd(12, 8), 4);
  assert.equal(algo.gcd(100, 75), 25);
  assert.equal(algo.gcd(7, 5), 1);
});

test('lcm', () => {
  assert.equal(algo.lcm(4, 6), 12);
  assert.equal(algo.lcm(3, 5), 15);
});

test('fibonacci', () => {
  assert.equal(algo.fibonacci(0), 0);
  assert.equal(algo.fibonacci(1), 1);
  assert.equal(algo.fibonacci(10), 55);
  assert.equal(algo.fibonacci(20), 6765);
});

test('prime_count', () => {
  assert.equal(algo.prime_count(10), 4);
  assert.equal(algo.prime_count(100), 25);
});

test('modpow', () => {
  assert.equal(algo.modpow(2, 10, 1000), 24);
  assert.equal(algo.modpow(3, 5, 13), 9);
});

test('is_prime', () => {
  assert.equal(algo.is_prime(2), true);
  assert.equal(algo.is_prime(17), true);
  assert.equal(algo.is_prime(4), false);
  assert.equal(algo.is_prime(1), false);
});

test('nqueens', () => {
  assert.equal(algo.nqueens(4), 2);
  assert.equal(algo.nqueens(8), 92);
});

// DP
test('max_subarray', () => {
  assert.equal(algo.max_subarray([-2, 1, -3, 4, -1, 2, 1, -5, 4]), 6);
  assert.equal(algo.max_subarray([1, 2, 3]), 6);
});

test('coin_change', () => {
  assert.equal(algo.coin_change([1, 5, 10, 25], 36), 3);
  assert.equal(algo.coin_change([2], 3), -1);
});

test('lis', () => {
  assert.equal(algo.lis([10, 9, 2, 5, 3, 7, 101, 18]), 4);
  assert.equal(algo.lis([1, 2, 3, 4, 5]), 5);
});

// String functions
test('lcs', () => {
  assert.equal(algo.lcs('abcde', 'ace'), 3);
  assert.equal(algo.lcs('abc', 'abc'), 3);
  assert.equal(algo.lcs('abc', 'def'), 0);
});

test('edit_distance', () => {
  assert.equal(algo.edit_distance('kitten', 'sitting'), 3);
  assert.equal(algo.edit_distance('', 'abc'), 3);
  assert.equal(algo.edit_distance('abc', 'abc'), 0);
});

// Array
test('array_sum', () => {
  assert.equal(algo.array_sum([1, 2, 3, 4, 5]), 15);
  assert.equal(algo.array_sum([0]), 0);
});

test('array_min / array_max', () => {
  assert.equal(algo.array_min([3, 1, 4, 1, 5, 9, 2, 6]), 1);
  assert.equal(algo.array_max([3, 1, 4, 1, 5, 9, 2, 6]), 9);
});

test('binary_search', () => {
  assert.equal(algo.binary_search([1, 3, 5, 7, 9, 11], 7), 3);
  assert.equal(algo.binary_search([1, 3, 5, 7, 9, 11], 4), -1);
});

test('is_sorted', () => {
  assert.equal(algo.is_sorted([1, 2, 3, 4]), true);
  assert.equal(algo.is_sorted([1, 3, 2]), false);
  assert.equal(algo.is_sorted([5]), true);
});

// Utility
test('djb2_hash', () => {
  const h = algo.djb2_hash('hello');
  assert.ok(typeof h === 'number', 'hash should be number');
  assert.notEqual(algo.djb2_hash('hello'), algo.djb2_hash('world'));
});

test('is_palindrome_num', () => {
  assert.equal(algo.is_palindrome_num(121), true);
  assert.equal(algo.is_palindrome_num(123), false);
  assert.equal(algo.is_palindrome_num(0), true);
});

test('digit_sum', () => {
  assert.equal(algo.digit_sum(123), 6);
  assert.equal(algo.digit_sum(999), 27);
});

test('bit_popcount', () => {
  assert.equal(algo.bit_popcount(7), 3);
  assert.equal(algo.bit_popcount(255), 8);
  assert.equal(algo.bit_popcount(0), 0);
});

// Knapsack: tests int64_t* pointer passing
test('knapsack', () => {
  // weights=[2,3,4], values=[3,4,5], capacity=7 → items 1+2: w=7, v=9
  assert.equal(algo.knapsack([2, 3, 4], [3, 4, 5], 7), 9);
});
