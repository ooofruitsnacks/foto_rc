# Welcome to Foto RC! 
Resize, convert, format or scale any picture simultaneously. 

## Features:
- Convert images between JPEG, PNG, or AVIF formats
- Convert image qaulity (10-100%)
- Convert image scale (10-100%)
- Convert image file size automatically, choose your desired file size (EX: "under 1 mb") and the program will adjust the quality, scale, and dimensions to meet the desired image file size.
- Set file size limits for conversions

# Setup and Installation:

open a new session in your preferred terminal and run:

```sh
git clone https://github.com/ooofruitsnacks/foto_rc.git
```

once it has finished downloading, run:

```sh
cd foto_rc
```

after changing directory to foto_rc, run:

```sh
cargo build --release
```

Once it has compiled you are free to use the program.

## !!! PLEASE KEEP IN MIND !!!

Changing the saved location of Foto RC will also change your path name. The path name can be found at the bottom of your viewport in finder (for MacOS only) Here is an example of the path being changed, make sure your path name is correct in the string.

original path:

<img width="414" height="54" alt="Screenshot 2026-07-15 at 1 13 45 PM" src="https://github.com/user-attachments/assets/34f41535-fb16-4dd1-8af5-d224aa06eaf9" />

updated path: 

<img width="465" height="36" alt="Screenshot 2026-07-15 at 1 14 01 PM" src="https://github.com/user-attachments/assets/c698ad26-6278-4c11-9414-69ee99e43753" />



# How to use Foto RC:

Go to the Foto RC directory

```sh
cd foto_rc
```

Once you are in the Foto RC directory, set a new "string". The string must include all the proper "flags". 

STRING EXAMPLE:

```sh
./target/release/foto_rc --input /users/example/screensavers/ --format png --quality 95 --scale 90
```

Let's breakdown this string to see how it works. The beginning of the string is the program.
```./target/release/foto_rc ```

The input flag is telling the program what directory/sub-directory/pciture to target. 
```--input /users/example/screensavers/``` 

If you want to target 1 photo, simply include the path to that one photo; like this below.
```--input /users/example/IMG_001/``` 

The next flags in this string command the program what to convert to. In this example we are telling the program to format the orginal images and convert them to PNG while converting the image quality down to 95% of the original image and scaleing down the image dimensions to 90% of the original image dimensions. 
```--format png --quality 95 --scale 90```

## How it works: Step by Step:

Original image/directory

<img width="506" height="248" alt="Screenshot 2026-07-15 at 1 20 15 PM" src="https://github.com/user-attachments/assets/6b35f90f-ce26-43a7-b0a1-1d76a118d059" />

Open Foto RC and Enter your String:

<img width="1136" height="228" alt="Screenshot 2026-07-15 at 1 20 52 PM" src="https://github.com/user-attachments/assets/06342229-83c6-4f7b-a7a1-7787c76f7796" />

Conversion Complete:

<img width="549" height="162" alt="Screenshot 2026-07-15 at 1 22 18 PM" src="https://github.com/user-attachments/assets/cd82dd92-e1c3-4588-bb34-fb9388728d79" />

Program automatically creates new folder so there's no confusion:

<img width="485" height="207" alt="Screenshot 2026-07-15 at 1 22 29 PM" src="https://github.com/user-attachments/assets/68769ef9-4331-437a-865b-7a05f9dee2cb" />

<img width="480" height="225" alt="Screenshot 2026-07-15 at 1 22 37 PM" src="https://github.com/user-attachments/assets/30f96cad-0144-4522-bf9a-a43ef6edd59f" />




## Flags

You can use the full name or an abbrievated version with just "-" and the beginning letter of the word. 

```--input, -i``` : target/source images

```--output, -o``` : output destination ( default: "<input>/resized" )

```format, -f``` : convert to different format( JPEG/JPG or PNG or AVIF or unchanged )

```--quality, -q``` : convert image quality ( min. 10% / max.100% ) if ```--target-size``` is used then ```--quality``` is ignored

```--scale, -s``` : scale down/up the converted image dimensions

```--target-size, -t``` : select a required file size, program automatically adjusts format, qaulity, and scale, to meet the file size limit. EX: You need the file under 1.5 MB, the program will reduce all other settings until the targeted file size is met. ( min. 500 KB / max. 1.5MB )

```--recursive, -r``` : include sub-directories/sub-folders in the target string

```--overwrite``` : Overwrite existing output file names on original picture

## Examples of uses: 

Convert all images to AVIF at 80% quality:
```./target/release/foto_rc --input ~/Photos --format avif --quality 80```

Scale all images to 50% of their original dimensions at 90% quality, keep original format:
```./target/release/foto_rc --input /users/example/pics/ --scale 50 --quality 90```

Convert all photo to under 500KB (auto-adjusts quality/scale as needed), converting to JPEG format:
```./target/release/foto_rc --input /users/example/pics/ --format jpeg --target-size 500kb```

Convert everything under 10KB:
```./target/release/foto_rc --input /users/example/pics/ --format jpeg --target-size "under 10kb"```

Cap the conversion file size at 1.5MB, include all sub-directories/sub-folders:
```./target/release/foto_rc --input /users/example/ --recursive --target-size 1.5mb```



### Target Size Mode:

Target Size Mode focuses on meeting your target file size. It will automatically adjust quality levels from 95% down to 10% at full image scale initially. If the image still can't meet targeted size, the program steps dimensions down from 100%, in steps of 10% and retries all quality levels at each size, stopping as soon as the image meets the targeted file size. Even if the most aggressive settings can't meet your target, it saves the smallest version it found and prints a warning for that file.



