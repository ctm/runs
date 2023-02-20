# ctm runs

This is an incredibly poorly written hack to extract finish times from
event results and then sum the times for the people who have finished
all of the events specified on the command line.

In theory, this could help out the Mt. Taylor 50k organizers who are
also in charge of awarding out scarves for
[doublers](http://www.mttaylor50k.com/mt.-taylor-doubler.html).  In
reality, at least in 2023, I took too long to bring this code up to
date to be a direct help.

Similarly, in theory, this could be useful for computing combined
times for the [Burque Brewery
Tour](https://www.abqroadrunners.com/burque-brewery-tour.html#/) or
the Albuquerque Road Runners Race Series (RIP, killed by covid), but
everything got weird in 2020 and I stopped work on this app.  When I
went to look at it again, I was mortified by the various Rust
atrocities I committed when I first wrote this code.
[Nom](https://docs.rs/nom/latest/nom/) is awesome, but my use of it
was putting together a jigsaw puzzle with a hammer.

Here's what it looks like to compute the 2022/2023 doublers:

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

Whee!
