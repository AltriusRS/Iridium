# Iridium Quick Start Guide
Welcome to the Iridium quick-start guide. In this guide you will learn how to use the tool in an efficient manner, which provides the best chances of success at all times!

# Basic Conversion
Okay, so you have just installed Iridium, and you want to convert a single file to test it, Lets have a look at how you would do that.

```sh
iridium -i ./path/to/file.md -o ./path/to/conversion.html
```

This command will automatically take the contents of the file at `./path/to/file.md` and convert it to the HTML file located at `./path.to/conversion.html`.

# Converting a directory
Iridium is smart, it will automatically detect if you are trying to convert a single file, or an entire directory. It will switch modes automatically, so you dont need to worry about specifying any flags here outside of the normal `-i` and `-o` flags for locating things.
Here is an example of how you would convert a directory:
```sh
iridium -i ./path/to/directory -o ./path/to/converted-directory
```


# Ignoring Files
I kept this in mind when writting Iridium, It is capable of accepting Gitignore style Ignorance rules, meaning you can just pass it a link to your .gitignore and it should work fine. (please report any problems via an issue)
```sh
iridium -i ./path/to/directory -o ./path/to/converted-directory --ignore ./path/to/ignore.file
```

# How do i change the theme?
There are four themes available at the moment:
- Iridium (the default theme)
- Iridium Light (the light version of the Iridium theme)
- Noir (A black and white theme entirely dedicated to being simplistic)
- Neon (A Neon themed adventure through the trippy world of purple and pink, Produced by [Jas777](https://github.com/Jas777)

```sh
iridium -i ./path/to/directory -o ./path/to/converted-directory --theme iridium-light
```

# How do i remove the watermark from the bottom of my files?
I am not an asshole, so you can remove it by adding the `--no-water-mark` flag to the commmand you run
