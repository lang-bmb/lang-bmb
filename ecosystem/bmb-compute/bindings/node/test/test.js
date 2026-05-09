'use strict';
const { test } = require('node:test');
const assert = require('node:assert/strict');
const c = require('..');

test('abs', () => {
  assert.equal(c.abs(-5), 5);
  assert.equal(c.abs(3), 3);
  assert.equal(c.abs(0), 0);
});

test('min / max / clamp', () => {
  assert.equal(c.min(3, 7), 3);
  assert.equal(c.max(3, 7), 7);
  assert.equal(c.clamp(5, 1, 10), 5);
  assert.equal(c.clamp(-5, 1, 10), 1);
  assert.equal(c.clamp(15, 1, 10), 10);
});

test('sign', () => {
  assert.equal(c.sign(-5), -1);
  assert.equal(c.sign(0), 0);
  assert.equal(c.sign(7), 1);
});

test('ipow / sqrt / factorial', () => {
  assert.equal(c.ipow(2, 10), 1024);
  assert.equal(c.sqrt(16), 4);
  assert.equal(c.sqrt(15), 3);
  assert.equal(c.factorial(5), 120);
  assert.equal(c.factorial(10), 3628800);
});

test('is_power_of_two / next_power_of_two', () => {
  assert.equal(c.is_power_of_two(8), true);
  assert.equal(c.is_power_of_two(6), false);
  assert.equal(c.next_power_of_two(5), 8);
  assert.equal(c.next_power_of_two(8), 8);
});

test('random', () => {
  const s0 = c.rand_seed(42);
  const s1 = c.rand_next(s0);
  assert.ok(s1 !== s0, 'state should change');
  const v = c.rand_pos(s0);
  assert.ok(v > 0, 'rand_pos should be positive');
  const r = c.rand_range(s0, 100);
  assert.ok(r >= 0 && r < 100, `rand_range out of bounds: ${r}`);
});

test('stats - sum / mean / min / max / range', () => {
  const arr = [1, 2, 3, 4, 5];
  assert.equal(c.sum(arr), 15);
  assert.equal(c.mean_scaled(arr), 3000);  // 3.000 × 1000
  assert.equal(c.min_val(arr), 1);
  assert.equal(c.max_val(arr), 5);
  assert.equal(c.range_val(arr), 4);
});

test('stats - variance / median', () => {
  const arr = [2, 4, 4, 4, 5, 5, 7, 9];
  // mean = 5, variance = 4
  assert.equal(c.variance_scaled(arr), 4000000);  // 4.000000 × 10^6
  // median of [2,4,4,4,5,5,7,9] = 4.5 → 4500
  assert.equal(c.median_scaled(arr), 4500);
});

test('magnitude_squared', () => {
  assert.equal(c.magnitude_squared([3, 4]), 25);   // 3²+4²=25
  assert.equal(c.magnitude_squared([1, 0, 0]), 1);
});

test('dot_product / dist_squared / weighted_sum', () => {
  assert.equal(c.dot_product([1, 2, 3], [4, 5, 6]), 32);   // 4+10+18
  assert.equal(c.dist_squared([0, 0], [3, 4]), 25);          // 9+16
  assert.equal(c.weighted_sum([1, 2, 3], [10, 20, 30]), 140); // 10+40+90
});
