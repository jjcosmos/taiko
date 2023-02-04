Taiko is a midi-to-json tool created for the purpose of authoring beatmaps for the VR game Ragnarock.

Usage is simple - just run taiko.exe and give it a source path to your midi file, and an output path for your .dat file.
For example: `./taiko.exe my_score.mid  my_beatmap_dir/Easy.dat`

Add the exe to you path if you want to be able to run it from anywhere without needing to use the entire path to the executable.

This tool is not meant to replace the beatmapping tool [Edda](https://github.com/PKBeam/Edda), but to work alongside it. I find authoring the maps in a DAW like Reaper to be easier, so this just creates a way to export that work into a readable format. At the moment, taiko does not generate an entire folder with all of the required components like info.dat, so make sure that the destination file is one of the Easy/Med/Hard files in your beatmap's directory. I would recomment backing up your work before overwriting any work, as this tool is still a WIP.