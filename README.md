# ctm runs

This software extracts finish times from event results and then scores them
by one of two methods.

I started writing it in February of 2019, before I began work on
[mb2](https://ctm.github.io/docs/players_manual/).  As such, much of
the ancient code quality is poor.  Perhaps my more recent code is OK.

This software can help out the Mt. Taylor 50k organizers who are also
in charge of awarding out scarves for
[doublers](http://www.mttaylor50k.com/mt.-taylor-doubler.html).  It can
also help the organizer of the [Albuquerque Road Runners Race Series](https://www.abqroadrunners.com/member-race-series.html).

## Duration Sum Mode

If this software's arguments are N plain files, it attempts to extract
finish times from each of the files and then computes the total time
spent in all N races for any entrant who finished all N races.

Here are the 2022/2023 doublers:

```
[master]% cargo r -- assets/mt_taylor_50k/2022.mhtml assets/quad/2023.mhtml
   Compiling runs v0.1.0 (/Users/ctm/runs)
    Finished dev [unoptimized + debuginfo] target(s) in 1.59s
     Running `target/debug/runs assets/mt_taylor_50k/2022.mhtml assets/quad/2023.mhtml`
  Patrick Goschke 10:05:48.1  4:54:43.0 5:11:05.1
      John Dailey 10:22:41.0  5:57:34.0 4:25:07.0
  Sean Krispinsky 10:36:22.7  5:58:49.0 4:37:33.7
  Mike Engelhardt 10:41:36.3  6:11:12.0 4:30:24.3
  Christian Ricks 11:16:53.0  6:41:28.0 4:35:25.0
     Brian Hutsel 11:42:13.9  6:16:45.0 5:25:28.9
      Andrew Gray 12:23:59.4  6:23:21.0 6:00:38.4
Clifford Matthews 13:38:09.6  7:08:23.0 6:29:46.6
 Joshua Rutkowski 14:06:22.7  8:02:45.0 6:03:37.7
   Richard Jensen 14:13:45.7  7:19:19.0 6:54:26.7
    Barry Roberts 14:35:56.8  8:50:33.0 5:45:23.8
   Douglas Deming 14:45:19.9  8:50:33.0 5:54:46.9
        Adam Delu 14:49:15.7  7:49:50.0 6:59:25.7
   Tiona Eversole 14:54:08.0  7:51:08.0 7:03:00.0
    Frank Novotny 15:19:20.6  8:07:44.0 7:11:36.6
Armando Mondragon 15:23:01.0  8:08:24.0 7:14:37.0
    Thondup Saari 15:25:46.6  8:09:45.0 7:16:01.6
     Ivy Crockett 15:31:53.3  7:50:04.0 7:41:49.3
  Nicole Highfill 15:34:09.6  8:40:16.0 6:53:53.6
 David Littlewood 15:45:33.2  8:59:55.0 6:45:38.2
   Collette Haney 16:22:46.7  9:14:41.0 7:08:05.7
     Aaron Reilly 16:34:52.2  9:25:22.0 7:09:30.2
      Rob Alunday 16:53:24.2  9:25:19.0 7:28:05.2
    Emily Novotny 17:04:19.0  8:02:32.0 9:01:47.0
   Valerie Denton 17:08:52.1 10:01:24.0 7:07:28.1
   Peter Mitchell 17:16:58.2  9:23:07.0 7:53:51.2
    Allison Mills 18:06:45.8 11:05:09.0 7:01:36.8
     Kevin Bishop 18:56:09.2  9:41:35.0 9:14:34.2
```
And here's everyone who has run all the Mt. Taylor 50ks:
```
[master]% cargo r -- assets/mt_taylor_50k/*
   Compiling runs v0.1.0 (/Users/ctm/runs)
    Finished dev [unoptimized + debuginfo] target(s) in 1.60s
     Running `target/debug/runs assets/mt_taylor_50k/2012.json assets/mt_taylor_50k/2013.json assets/mt_taylor_50k/2014.json assets/mt_taylor_50k/2015.json assets/mt_taylor_50k/2016.json assets/mt_taylor_50k/2017.json assets/mt_taylor_50k/2018.json assets/mt_taylor_50k/2019.json assets/mt_taylor_50k/2021.mhtml assets/mt_taylor_50k/2022.mhtml`
    Barry Roberts 65:11:23.0  5:57:45.0 5:47:24.0 6:02:18.0 6:19:20.0 6:09:48.0 6:22:22.0  6:23:13.0  6:26:57.0  6:51:43.0  8:50:33.0
Clifford Matthews 72:39:24.0  7:22:14.0 7:23:29.0 7:18:44.0 8:14:00.0 8:11:56.0 7:12:15.0  6:13:03.0  6:52:16.0  6:43:04.0  7:08:23.0
      Randy Silva 86:02:29.0  7:53:10.0 7:45:21.0 8:06:05.0 8:46:32.0 8:40:29.0 8:47:55.0  8:29:48.0  8:47:31.0  9:09:02.0  9:36:36.0
      Eddie Dimas 88:01:41.0  8:16:25.0 8:34:27.0 9:49:22.0 9:18:16.0 7:53:02.0 8:33:01.0  7:12:11.0  7:27:57.0 10:55:38.0 10:01:22.0
 Crystal Anderson 93:58:41.0 10:10:36.0 7:56:02.0 7:52:28.0 8:03:23.0 7:46:30.0 9:52:52.0 10:09:43.0 10:48:08.0 10:02:44.0 11:16:15.0
```

## Albuquerque Road Runners 2023 Race Series Point Modes

### Points

From the [Albuquerque Road Runners Member Race Series Page](https://www.abqroadrunners.com/member-race-series.html):

> For each race you finish, you’ll score points based on the winner’s
> time divided by your time and expressed as a percentage. For
> example, the winner finishes in 45:00 and you run 60 minutes, your
> score is 75 points (45/60 * 100); male and female results calculated
> separately.

> There are 6 categories of race: standard 5k, 10, half-marathon and
> marathon distances plus short trail and long trail races, and to
> encourage people to try different things only your best score in
> each category counts for the year-end total. As before, 10-year
> age/gender divisions will apply, with your age on July 1 2023
> determining which age-group you fall into.


### Category Mode


If a single directory is supplied as a command line argument, and the
contents of that directory are all files, then that directory is considered
a series category (e.g. 5k, 10k) and points are calculated for all the
participants, with one line created for each participant.  That line has
the maximum number of points for that participant and the race (or one
of the races if there's a tie) that the participant gained those points
from.

Here's an example of the top twenty point scorers using the 2023 scoring
system on the 2022 results of the 2023 10k races:

```
[master]% cargo r assets/abq_rr/2022/10k | head -20
    Finished dev [unoptimized + debuginfo] target(s) in 14.21s
     Running `target/debug/runs assets/abq_rr/2022/10k`
100: Jeff Cuno                 Great Pumpkin Chase 10k 2022
100: Katherine Lindenmuth      Run to Break The Silence 10k 2022
100: Kellie Nickerson          Great Pumpkin Chase 10k 2022
100: Tyrell Natewa             Run to Break The Silence 10k 2022
100: Emily Boles               Shamrock Shuffle 10k 2022
100: Chris Bratton             Shamrock Shuffle 10k 2022
 99: Christopher Bratton       Great Pumpkin Chase 10k 2022
 99: Jaime Dawes               Great Pumpkin Chase 10k 2022
 99: Erin Castillo             Shamrock Shuffle 10k 2022
 97: Caiden Dawes              Great Pumpkin Chase 10k 2022
 95: Ryan Platz                Great Pumpkin Chase 10k 2022
 93: Veronica Hutchinson       Run to Break The Silence 10k 2022
 93: Athena Shapiro            Great Pumpkin Chase 10k 2022
 93: Lynette Trujillo          Great Pumpkin Chase 10k 2022
 91: Gianna Rahmer             Run to Break The Silence 10k 2022
 90: Cheryl Lowe               Great Pumpkin Chase 10k 2022
 89: David Hubbard             Great Pumpkin Chase 10k 2022
 89: Miranda Harrison Marmaras Great Pumpkin Chase 10k 2022
 89: Kara Cervantes            Shamrock Shuffle 10k 2022
 87: Sydney Billingsley        Shamrock Shuffle 10k 2022
```


### Series Mode

If a single directory is supplied as a command line argument, and the
contents of that directory are all directories, then the top-level directory
is considered a series, with the sub-directories each being categories.  As
such, complete series standings are computed and printed.

Here's an example of some of the output using the 2023 scoring but
with the 2022 results for the 2023 categories and races:

```
[master]% cargo r assets/abq_rr/2022 | head -23
cargo r assets/abq_rr/2022 | head -23
    Finished dev [unoptimized + debuginfo] target(s) in 0.16s
     Running `target/debug/runs assets/abq_rr/2022`
   1  398 Kellie Nickerson
      100   Great Pumpkin Chase 10k 2022
      100   Forever Young 6 Miler 2022
       98   Chocolate and Coffee 5k 2022
      100   Run For The Zoo Half Marathon 2022

   2  340 Anthony Phillips
       85   Duke City Marathon Full 2022
       63   Mt. Taylor 50k 2022
      100   Forever Young 6 Miler 2022
       92   Chocolate and Coffee 5k 2022

   3  331 Anthony Martinez
       87   Great Pumpkin Chase 10k 2022
       78   Duke City Marathon Full 2022
       83   Cherry Garcia 5k 2022
       83   Chips and Salsa Half Marathon 2022

   4  310 Christopher Brownsberger
       78   Great Pumpkin Chase 10k 2022
       79   Forever Young 6 Miler 2022
       79   Chocolate and Coffee 5k 2022
       74   Duke City Marathon Half 2022
...

  26  195 Zach Chenoweth
       99   Chunky Monkey 5k 2022
       96   Duke City Marathon Half 2022

  26  195 Samantha Nelson
       63   Shamrock Shuffle 10k 2022
       69   Sandia Mountain Shadows Trail Run 5k 2022
       63   Run For The Zoo Half Marathon 2022

  28  194 Michael Brown
       63   Shiprock Marathon 2022
       65   Chocolate and Coffee 5k 2022
       66   Chips and Salsa Half Marathon 2022

  29  193 Chris Bratton
      100   Shamrock Shuffle 10k 2022
       93   Chips and Salsa Half Marathon 2022

  29  193 Clifford Matthews
       52   Duke City Marathon Full 2022
       58   Mt. Taylor 50k 2022
       83   Forever Young 6 Miler 2022

  31  191 Ty Martinez
       57   Duke City Marathon Full 2022
       71   Chocolate and Coffee 5k 2022
       63   Chips and Salsa Half Marathon 2022
...
```
