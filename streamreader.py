from threading import Thread
from queue import Queue, Empty

class LineStreamReader:
    def __init__(self, stream, maxsize = 1000):
        self._stream = stream
        self._queue = Queue()
        self._maxsize = maxsize

        def _populateQueue(stream, queue, size):
            while True:
                line = stream.readline();
                if line:
                    queue.put(line)

                    while queue.qsize() > size:
                        try:
                            queue.get(block = False)
                        except Empty:
                            pass

        self._thread = Thread(target=_populateQueue, args = (self._stream, self._queue, self._maxsize))   
        self._thread.daemon = True
        self._thread.start()

    def getLine(self, timeout = None):
        try:
            return self._queue.get(block = timeout is not None, timeout = timeout)
        except Empty:
            return None

