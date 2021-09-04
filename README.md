Pirate Life
---

pirate_life is a straightforward program that "glues together" multiple other programs in a semi-sane manner to make managing downloads, tagging media, and re-encoding files requiring taking only command.

It requires you have ffmpeg, aria2c, and propwriter (my program) installed locally. 

The goal of pirate life is to manage the following workflow

1. Downloading a larger media file, handle chunking, retrying, and parallel operations
2. Re-encoding the file with essentially default settings
3. Applying windows metadata (tags, artists, producers, etc.)
4. Moving the file to a final location.
