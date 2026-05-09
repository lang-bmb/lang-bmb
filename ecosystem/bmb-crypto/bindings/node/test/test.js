'use strict';
const { test } = require('node:test');
const assert = require('node:assert/strict');
const c = require('..');

test('sha256', () => {
  const h = c.sha256('hello');
  assert.equal(h, '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824');
  assert.equal(c.sha256(''), 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855');
});

test('md5', () => {
  const h = c.md5('hello');
  assert.equal(h, '5d41402abc4b2a76b9719d911017c592');
});

test('base64', () => {
  const enc = c.base64_encode('hello');
  assert.equal(enc, 'aGVsbG8=');
  assert.equal(c.base64_decode(enc), 'hello');
});

test('base32', () => {
  const enc = c.base32_encode('hello');
  assert.ok(enc.length > 0, 'base32 should produce output');
  assert.equal(c.base32_decode(enc), 'hello');
});

test('hex_encode / hex_decode', () => {
  const enc = c.hex_encode('hello');
  assert.equal(enc, '68656c6c6f');
  assert.equal(c.hex_decode(enc), 'hello');
});

test('rot13', () => {
  assert.equal(c.rot13('hello'), 'uryyb');
  assert.equal(c.rot13(c.rot13('hello')), 'hello');  // involution
});

test('checksums (crc32 / adler32 / fletcher16 / xor_checksum)', () => {
  const crc = c.crc32('hello');
  assert.ok(crc.length > 0, 'crc32 should produce output');

  const adl = c.adler32('hello');
  assert.ok(adl.length > 0, 'adler32 should produce output');

  const fl = c.fletcher16('hello');
  assert.ok(fl.length > 0, 'fletcher16 should produce output');

  const xor = c.xor_checksum('hello');
  assert.ok(xor.length > 0, 'xor_checksum should produce output');

  // Different inputs should give different checksums
  assert.notEqual(c.crc32('hello'), c.crc32('world'));
});

test('hmac_sha256', () => {
  const h = c.hmac_sha256('secret', 'message');
  assert.ok(h.length === 64, `hmac_sha256 should be 64 hex chars, got ${h.length}`);
  // Deterministic
  assert.equal(c.hmac_sha256('secret', 'message'), h);
  // Different key → different output
  assert.notEqual(c.hmac_sha256('other', 'message'), h);
});
