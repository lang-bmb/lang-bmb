'use strict';
const { test } = require('node:test');
const assert = require('node:assert/strict');
const t = require('..');

test('str_find / str_rfind / str_count', () => {
  assert.equal(t.str_find('hello world', 'world'), 6);
  assert.equal(t.str_find('hello', 'xyz'), -1);
  assert.equal(t.str_rfind('abcabc', 'abc'), 3);
  assert.equal(t.str_count('abcabcabc', 'abc'), 3);
});

test('kmp_search', () => {
  assert.equal(t.kmp_search('hello world', 'world'), 6);
  assert.equal(t.kmp_search('hello', 'xyz'), -1);
});

test('str_contains / starts_with / ends_with', () => {
  assert.equal(t.str_contains('hello world', 'world'), true);
  assert.equal(t.str_contains('hello', 'xyz'), false);
  assert.equal(t.str_starts_with('hello', 'hel'), true);
  assert.equal(t.str_starts_with('hello', 'ell'), false);
  assert.equal(t.str_ends_with('hello', 'llo'), true);
  assert.equal(t.str_ends_with('hello', 'hel'), false);
});

test('find_byte / count_byte', () => {
  assert.equal(t.find_byte('hello', 'l'), 2);
  assert.equal(t.find_byte('hello', 'z'), -1);
  assert.equal(t.count_byte('hello', 'l'), 2);
});

test('is_palindrome / word_count / str_len', () => {
  assert.equal(t.is_palindrome('racecar'), true);
  assert.equal(t.is_palindrome('hello'), false);
  assert.equal(t.word_count('hello world foo'), 3);
  assert.equal(t.str_len('hello'), 5);
});

test('str_reverse / to_upper / to_lower / trim', () => {
  assert.equal(t.str_reverse('hello'), 'olleh');
  assert.equal(t.to_upper('hello'), 'HELLO');
  assert.equal(t.to_lower('HELLO'), 'hello');
  assert.equal(t.trim('  hello  '), 'hello');
});

test('str_replace / str_replace_all / repeat', () => {
  assert.equal(t.str_replace('hello world world', 'world', 'there'), 'hello there world');
  assert.equal(t.str_replace_all('hello world world', 'world', 'there'), 'hello there there');
  assert.equal(t.repeat('ab', 3), 'ababab');
});

test('hamming_distance / str_compare', () => {
  assert.equal(t.hamming_distance('karolin', 'kathrin'), 3);
  const cmp = t.str_compare('abc', 'abd');
  assert.ok(cmp < 0, 'abc < abd');
  assert.equal(t.str_compare('abc', 'abc'), 0);
});

test('token_count', () => {
  assert.equal(t.token_count('a,b,c,d', ','), 4);
  assert.equal(t.token_count('hello', ','), 1);
});
