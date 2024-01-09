#!/usr/bin/python
## add an entry to a packing list in Simple format, given a YouTube URL.
## copy this file into your album USBs.
import os
import sys

startout = os.listdir(".") ## really shitty solution to the problem of making yt-dlp work for me
os.system("yt-dlp -x --audio-format mp3 " + sys.argv[1])
now = os.listdir(".")
for x in now:
    if not x in startout:
        name = x.split("[")[0]
        formatted_name = name.lower().replace(" ", "").replace("(", "").replace(")", "").replace("'", "").replace("&", "and")
        print("mv \"" + x.replace("\"", "\\\"") + "\" " + formatted_name + ".mp3")
        os.system("mv \"" + x.replace("\"", "\\\"") + "\" " + formatted_name + ".mp3")
        packlist = open("packlist", "a")
        packlist.write(name + ", " + formatted_name + ".mp3" + "\n")
print("done")
