Taiko is a midi-to-json tool created for the purpose of authoring beatmaps for the VR game Ragnarock.

Ther are currently 2 different ways to use taiko.

One converts a midi file and outputs to the path of your choosing, i.e.
`taiko convert "path_to_midi_file.mid" "path_to_output_file.dat"`
Alternatively, taiko can parse a multi-track midi file for their track names, and use them as separate output files like so:
`taiko auto "path_to_midi_file.mid" "path_to_output_folder"`
Assuming you have tracks named "Easy", "Medium", and "Hard", taiko would generate "Easy.dat", "Medium.dat", and "Hard.dat".

Add the exe to you path if you want to be able to run it from anywhere without needing to use the entire path to the executable.

To interactively configure which midi values are mapped to which drum, or the output file format, run `taiko configure` 

I would also like to make those pitches easier to author in the future, but for now, something like [this](https://www.inspiredacoustics.com/en/MIDI_note_numbers_and_center_frequencies) can serve as a reference.

This tool is not meant to replace the beatmapping tool [Edda](https://github.com/PKBeam/Edda), but to work alongside it. I find authoring the maps in a DAW like Reaper to be easier, so this just creates a way to export that work into a readable format. It is important that the midi exported from your DAW contains metadata like tempo and time signature. 

At the moment, taiko does not generate an entire folder with all of the required components like info.dat, so make sure that the destination file is one of the Easy/Med/Hard files in your beatmap's directory. I would recommend backing up your work before overwriting any in-progress maps, as this tool is still a WIP.