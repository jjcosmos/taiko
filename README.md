Taiko is a midi-to-json tool created for the purpose of authoring beatmaps for the VR game Ragnarock.

![Screenshot](https://github.com/jjcosmos/taiko/blob/main/Screenshot.png)

## How To:

### File Output type (single/multi):
- Single - Takes a midi file and convert it to a single .dat file.
- Multi - Takes a midi file and parses out multiple tracks into .dat files corresponding to the track names.

### Source:
- The source midi file to convert. The midi file MUST contain metadata such as tempo and time signature - otherwise the results will be nonsense.

### Destination:
- The folder the .dat file/files are output to. As always, back up any work before overwriting.

### Configuration (Right Panel):
- Map drums to the midi pitches of your choosing. Save the config to have the app remember your choices.

*In the event your editor does not show midi pitch values, use something like [this](https://www.inspiredacoustics.com/en/MIDI_note_numbers_and_center_frequencies).*

## Disclaimer

This tool is not meant to replace the beatmapping tool [Edda](https://github.com/PKBeam/Edda), but to work alongside it. I find authoring the maps in a DAW like Reaper to be easier, so this just creates a way to export that work into a readable format. It is important that the midi exported from your DAW contains metadata like tempo and time signature. 

At the moment, taiko does not generate an entire folder with all of the required components like info.dat, so make sure that the destination folder is in your beatmap's directory. I would recommend backing up your work before overwriting any in-progress maps, as this tool is still a WIP. 

*Because it does not generate info.dat, make sure bpm is manually set in edda, as it seems to ignore a bpm change at global beat 0.*

#### Like what I'm doing?
<a href='https://ko-fi.com/jjcosmos' target='_blank'><img height='35' style='border:0px;height:34px;' src='https://az743702.vo.msecnd.net/cdn/kofi3.png?v=0' border='0' alt='Buy Me a Coffee at ko-fi.com' />
