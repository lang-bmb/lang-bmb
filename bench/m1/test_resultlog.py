"""Results files written as sessions finish, and resumed (backlog INF-16)."""
import json
import os
import stat
import sys
import tempfile
import threading
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import resultlog  # noqa: E402
from resultlog import ResultLog  # noqa: E402

RUN = {"agent": "openai:m", "task_set": "m1", "manifest": "d", "commit": "c", "tasks": ["a", "b", "c"]}


def lines(path):
    with open(path) as f:
        return [json.loads(line) for line in f]


class Log(unittest.TestCase):
    def setUp(self):
        self.path = os.path.join(tempfile.mkdtemp(), "r.jsonl")

    def test_rows_are_on_disk_as_sessions_finish(self):
        log = ResultLog(self.path, RUN)
        log.add({"id": "b", "success": True})
        self.assertEqual(lines(self.path), [{"partial": True, **RUN}, {"id": "b", "success": True}])
        self.assertEqual(resultlog.status(self.path), ("partial", {"partial": True, **RUN}))

    def test_resume_skips_written_rows_and_ends_like_one_run(self):
        log = ResultLog(self.path, RUN)
        log.add({"id": "b", "n": 2})
        with open(self.path, "a") as f:
            f.write('{"id": "c", "n"')  # cut off mid-write: that session left no row
        log = ResultLog(self.path, RUN)
        self.assertEqual((log.done("a"), log.done("b"), log.done("c")), (False, True, False))
        self.assertEqual(log.resumes, 1)
        log.add({"id": "a", "n": 1})
        log.add({"id": "c", "n": 3})
        rows = [log.rows[t] for t in RUN["tasks"]]
        log.finish(rows, {"summary": True})
        self.assertEqual(lines(self.path), [{"id": "a", "n": 1}, {"id": "b", "n": 2}, {"id": "c", "n": 3},
                                            {"summary": True, "resumes": 1}])
        self.assertEqual(resultlog.status(self.path)[0], "finished")

    def test_resumes_are_counted(self):
        ResultLog(self.path, RUN)
        ResultLog(self.path, RUN)
        self.assertEqual(ResultLog(self.path, RUN).resumes, 2)

    def test_uninterrupted_run_has_no_resume_field(self):
        log = ResultLog(self.path, RUN)
        for t in RUN["tasks"]:
            log.add({"id": t})
        log.finish([log.rows[t] for t in RUN["tasks"]], {"summary": True})
        self.assertEqual(lines(self.path)[-1], {"summary": True})

    def test_refuses_finished_and_other_runs(self):
        log = ResultLog(self.path, RUN)
        for other in ({**RUN, "commit": "c2"}, {**RUN, "agent": "anthropic:m"}, {**RUN, "tasks": ["a"]}):
            with self.assertRaises(SystemExit):
                ResultLog(self.path, other)
        log.finish([], {"summary": True})
        with self.assertRaises(SystemExit):
            ResultLog(self.path, RUN)
        with open(self.path, "w") as f:
            f.write('{"id": "a"}\n')  # not a results file of this module
        with self.assertRaises(SystemExit):
            ResultLog(self.path, RUN)

    def test_an_empty_file_is_a_new_run(self):
        open(self.path, "w").close()
        ResultLog(self.path, RUN)
        self.assertEqual(lines(self.path), [{"partial": True, **RUN}])

    def test_without_a_path_rows_stay_in_memory(self):
        log = ResultLog(None, RUN)
        log.add({"id": "a"})
        log.finish([log.rows["a"]], {"summary": True})
        self.assertTrue(log.done("a"))

    def test_a_device_or_pipe_is_written_through_not_replaced(self):
        # `--out /dev/null` once renamed a regular file over /dev/null.
        fifo = self.path + ".fifo"
        os.mkfifo(fifo)
        got = []
        reader = threading.Thread(target=lambda: got.extend(drain(fifo)))
        reader.start()
        log = ResultLog(fifo, RUN)
        reader.join()
        reader = threading.Thread(target=lambda: got.extend(drain(fifo)))
        reader.start()
        log.add({"id": "a"})
        reader.join()
        reader = threading.Thread(target=lambda: got.extend(drain(fifo)))
        reader.start()
        log.finish([log.rows["a"]], {"summary": True})
        reader.join()
        self.assertTrue(stat.S_ISFIFO(os.stat(fifo).st_mode))
        self.assertFalse(os.path.exists(fifo + ".tmp"))
        self.assertEqual(got[-2:], [{"id": "a"}, {"summary": True}])


def drain(path):
    with open(path) as f:
        return [json.loads(line) for line in f if line.strip()]


if __name__ == "__main__":
    unittest.main()
