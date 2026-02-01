---
id: src_01KGAMQP2QCND16S0Q271PWVCJ
title: 2025-01-23-chris-mondoo-agentic
ingested_at: 2026-01-31T18:23:38.071293+00:00
original_path: /Users/jesse/cto/meetings/2025-01-23-chris-mondoo-agentic.md
tags: []
processing_status: complete
content_hash: sha256:c0edf3a95645034057c5d3852d56732a5f35d493cdbb16fa409fcb01f1cc6fc7
---
# Chris (Mondoo) - Agentic Development Discussion

**Date:** January 23, 2025
**Participants:** Jesse Andrews, Chris (Mondoo CTO/VPE)
**Granola link:** https://notes.granola.ai/t/ab6acd78-5b65-4181-844c-919d609b43fb

---

## Clarifications (post-meeting)

**Verification loop:**
- Scripts at `~/lw/sparks/scripts/verify/` - tests networking, mosh, console, git projects, snapshots, ephemeral instances, data, etc.
- Currently ~25 min for full verification across components

**Multi-run management:**
- Currently manual (terminal tabs)
- Sparks is being built partly to provide better dev environment for parallelization
- Goal: tools that manage multiple concurrent agent runs

**Gemini PR review prompts:**
- Currently ad-hoc
- Skills repository work (see Carl sync same day) will systematize prompt/skill definitions
- Plan: shared `loop.work/skills` repo with consolidated definitions

**Spec → variations approach:**
- Done via: Lluminate-style evolutionary runs, manual prefacing with different words, varying details sent
- Want: Sparks tooling to help systematize this

**Key model personality difference:**
- Codex: runs until done (can go 4+ hours)
- Claude: takes breaks, thinks in smaller increments, needs context management

---

## Summary

### Mondoo's Current AI Adoption

- Fully rewritten development workflow since Claude 4.5 release (December)
- Complete website migration from Webflow to Next.js in 2 weeks
  - Marketing team now updates site via Slack through Claude
  - No CMS - git repository + Claude abstractions for content
  - Fixed 5,000 SEO errors within a week post-launch
  - Mondoo app indexing dramatically improved
- Customer success team using Claude Code to avoid engineering dependencies
- UI team onboarding to Claude Code for faster iteration
- Backend migration more cautious due to customer data risks
- Policy development in MQL becoming fully autonomous

### Jesse's Multi-Agent Development Approach

**Primary tools:** Claude Code + Codex CLI (ChatGPT)
- Codex better for architectural work, runs 20-30 minutes unattended
- Claude superior for UI, DevOps, shell scripting
- Co-founder (Carl) uses Amp (pay per use, auto-switches to best model)

**Model selection:**
- Codex 5.2 High for implementation (not Extra High - too slow, budget drain)
- GPT 5.2 Extra High for spec analysis and clarifying questions
- Gemini for large context analysis tasks

**Cost philosophy:** Expensive thing is now human review time, not model time

**Multi-run approach:**
- Run multiple agents simultaneously rather than micromanaging single instance
- If one fails, discard sandbox and restart
- Probability of same failure twice very low
- Treat runs as experiments, not golden PR attempts

### Technical Implementation

**Sandboxing:**
- All agents run in containers/VMs with limited blast radius
- Use `dangerously_allow_all` + `IS_SANDBOX=1` flags for YOLO mode
- Git access limited to specific branches

**RALPH loop concept:** Simple while-true loop until task completion
```
while not complete:
    create new Claude instance
    prompt = spec + "find next incomplete task, work on it"
    run until stopping point
    if git tree not clean: prompt to commit
    run lint/unit/integration tests
    provide feedback if failing
```

**Context management:**
- Claude goes into "just get done" mode above 50% context
- Plan mode helps break large tasks into smaller context windows
- Codex compaction actually works - can run 4+ hours on complex tasks

### Verification and Quality Control

**25-minute verification loops:**
- Deploy to full GCP/K8s clusters
- Verify important flows actually work end-to-end

**Multi-model code review:**
- Gemini generates systematic PR documentation (large context advantage)
- Prompt: "Write for people who built this, explain what changed and why"
- Critical review required when verification steps change
- Five parallel implementations for important backend work
- Generate "blog post style" explanations of implementations

**Tools mentioned:**
- StageHand (BrowserBase) - LLM-driven visual testing without brittle DOM selectors
- Augmentation tools for component annotation and live feedback

**Goal:** Move from "golden PRs" to understanding system evolution

### Philosophical Discussion

**Engineering hasn't fundamentally changed in 10 years until now:**
- Shift from implementation focus to problem definition
- Iteration cycles dramatically faster
- "The technology is just the means to get there"
- Value is in the product offering, not the implementation

**Mental model shift:**
- Before: Optimize for minimal changes (cost of change was high)
- Now: Why optimize when you can build what you want and refactor if needed?
- Marketing example: Agency iteration was slow prompt→implement→wrong→repeat
- Now: Instant feedback, ship imperfect, iterate tomorrow

**Risks for large enterprises:**
- Everyone who can't code now has Claude Code
- 20,000 small disconnected systems nobody understands
- Security and maintenance nightmares
- "We can't explain the system anymore" - similar to LLM interpretability problem

**Junior engineer pipeline concern:**
- How do juniors get experience if no hiring pipeline?
- Counter: Maybe nobody needs low-level knowledge anymore (like assembler)
- Abstractions keep getting better

### Programming Language Evolution

- All great PL ideas existed since 70s/80s but chicken-egg adoption problem
- RL training breakthrough (DeepSeek comparative ranking) enables faster iteration
- Prediction: Explosion of languages designed LLM-friendly from the beginning
- LLMs can analyze shortcomings of existing languages and design better ones

### Next Steps

- Chris to try RALPH looping approach with verification steps
- Explore StageHand for UI testing automation
- Test Augmentation tool for better component feedback
- Consider MQL policy development in natural language
- Follow up meeting in ~2 weeks

---

## Key Quotes

**On micromanagement:**
> "My goal is always to be like, I want the equivalent of meeting with tech lead of another team where I'm like, I trust you could implement a JSON schema. We don't have to talk about that level."

**On verification ownership:**
> "If they own the code, they own the unit tests... me trusting the unit test to say 'yeah I should merge that' is silly because they could make it return true."

**On probabilistic nature:**
> "Everybody outside, every time anybody's like 'oh check all this cool thing', they're just one-shotting it... If this is important, these are probabilistic machines."

**On Claude's failure mode:**
> "If it gets above 50% context window, Claude just goes like, I need to just get done any way possible, including just lying."

**On the fundamental shift:**
> "Engineering has not developed for the last 10 years before Claude Code... conceptually nothing has changed. Now you essentially move the mental cycle more on what's the problem, what we solve, and not on how."

---

## Raw Transcript

[Full transcript preserved below for reference]

<details>
<summary>Click to expand full transcript</summary>

Me: They just have a doc I could share with people for the basics, and that way I could move on to the fun stuff.
Them: Yeah, that makes a lot of sense.
Me: Okay?
Them: Are you doing? Better. Sue mentioned you sick.
Me: I am still sick. The cough is about a month old now, and I'm hoping, though, in the next. I'm having a CT scan soon, so hopefully that will help.
Them: Okay?
Me: Yeah. So sue shared that you've been using Claude code for web stuff.
Them: I'm not just using Clockworks. I use cloud, like, as my main coding thing.
Me: Oh, okay, cool.
Them: I don't use it. I rewritten. Essentially, since one of it. Before Christmas. I think this was like after Oppo's 4.5 came out, so the quality went up dramatically.
Me: Exactly.
Them: And so I started building. I wanted to build a website, and I was, like, surprised how. How good it is. So it was really good. And so then I talked with Dom and Patrick and then we said, like, our website is shit. And then essentially what we've done is two weeks, we converted the whole website from fucking webflow over to over to next JS with latest fancy thingies, everything on versa. Marketing team is talking to Claude now via Slack to update things on the website, so.
Me: Wow. Nice. Lots of changes.
Them: That's all. So for the, for this team, we essentially like the home content. CMS stuff has like removed. We do this over git repository and Claude is doing the abstractions. So we decided not to do any CMS because then cloud has no access. So no is relative but like more difficult.
Me: Yeah.
Them: And it helped us dramatically. So, like in C. O, for example, we fixed, like, after we flipped to the new site, and we flipped 5,000 arrows, like, super quick, win a week. So index for mondw is app now. So much better. So. So it was really good. It's really helpful. We introduced this to the customer success team so they do now staff all with cloud code so they don't have to wait for engineering. I'm currently trying to. Onboard the whole UI team on Claude codes, so they don't take ages for a button. So the uis I think is in. For example, I was like very frustrated that we're still working with a single page app. So I'm talking about switching to Next JS for two years now and had not leveraged nor the time to do it. Claude has done that over a weekend for me, so. So that's. I'm a little bit more careful on the backend side. So we need to migrate data and customer data. But new features are being cranked out already also with cloud code. And the resource development in MQL is now. I think since we have enough, it's like. Good enough that Claude can infer, like, how, how new resources are being written. So policy development is coming to be fully autonomous. I would say, like, very close to. So I would say it's not just like, web. I would say it's like full engineering. So the. So the main question that I had was sue. Mentioned, like, you have multiple agents running in parallel, and they do a lot of stuff on the site. Just want to understand, like, how your setup is. So that right now it feels I'm working a little bit on the stepping stone age online cloud code because I still have to, like, I have five parallel agents running, but I still need to, like, somehow say yes, no, yes, no. But I hate son like this to, like, optimize that. So want to understand, like, what you're using.
Me: Cool. Yeah. Yeah. Yeah. Cool. Yeah, definitely. I would say almost everything you said. I use Claude and Codex. They're my two primary ones. My co founder uses amp which you pay per use, but he finds it extremely valuable. And it wraps whatever the best model is. So, like, this December, it was like, Gemini comes out. So, like, our default model is Gemini for three days. And then Opus 4. 5 came out, and it's like, our default model is this. Yeah. So they kept on switching. I agree 100% that December 5th, like December 14th. 15th, I forget at this point was a watershed where it's like, okay, the models are now good enough. Where people that have one of my good friends from planet who's doing finite element analysis and low level math and physics stuff. And he's like, it's actually a better C programmer than he is. And so he's like, I can now just turn it over to it. Similarly, I know a lot of languages, but I'm not an expert at rust. But Rust gives you a lot of benefit, and so our startup uses Rust as his primary language. I could read Rust if you put me. Put a gun to my head and ask me to write hello, World. I probably couldn't because remembering I don't have that. Like I have for so many other language. The active finger memory of like how to type a new file.
Them: Yep.
Me: But because you're saying you're starting to get into back end, I would highly recommend trying Codex the Chat GPT1. I find that. For architectural work. It actually does two things very different than Claude. One is, it runs for a much longer time. It means that you actually have to work with it a different way. You like, start your chatgpt going your codex client go and then come back to it 20, 30 minutes later. You don't sit there and micromanaging it. And it's nowhere near as good as Claude at UI or like fast help me being a sort of in between between you and shell scripts and things like that. Claude is much better at helping me with the DevOps or automating FFmpeg. That level of complexity. I know what I need to do, but remembering the actual flags. FFmpeg is like, that's a Sisyphean tax.
Them: Yep. Agreed. Agree. D. Yeah, no, I think. I think that's. That's. That matches my perception, I think. Like, when you say codex, like, so are you developing, like, architectural setups and then you give it to Claude for implementation?
Me: No, I still stick with two things. Okay, so there's a lot in that question. So I usually find it's more successful. ChatGPT Codex CLI specifically I don't use the web version, I use the Codex CLI is a stronger implementer when it comes to fix like Backendy or Clie stuff for me in practice. Every project and language and things like that will change that though. And the rumors are in the next week we will have new versions of all three models. Gemini, ChatGPT and Opus supposedly, or sorry, and Clotter supposedly coming out with new models in the next week or two, which are even better. So what model is best constantly changes. Which brings me to the other point, which is why run one when you could run three at three times the cost? The expensive thing is starting to become your time, not the model's time even, because the quality that it could produce versus the cost of what it can produce is. And the cost to you. To review it or understand if it's the right thing is what you need to manage towards, not what's moved. When I think about the phases of leveraging these agentic coding systems, going from it's a toy to tap completion to using cloud code as a single one, and then using multiple of them and keeping them aligned and then going actually, I don't want to micromanage clauds. That sounds worse than yes, it's giving me a lot of value, but it becomes a bottleneck, and you're constantly in this sort of micromanaging mode rather than being able to stay at the high level. And my goal is always to be like, okay, I want the equivalent of meeting with tech lead of another team where I'M like, I trust you could implement a JSON schema. We don't have to talk about that level. Let's talk about what and why and how and these big level things. And similarly, I don't want to be saying yes and no to individual things. What I want to do is create a system that allows it to iterate without me having to be there to say yes and no. Starting to think about these runs? As why not run five of them? Why not run 10 of them? Especially if you could do them in parallel and because what happens is if it goes wrong. Do you know the pink elephant problem? Like when she mentioned a pink elephant, you can't stop thinking about. This was very clear on image models for the longest time. Where if you say no beard, there would be a beard because you said the word beard. And LLMs have that problem still encoding agents where it's like a few bad words or one bad path. Can make it be the difference between an amazing session and a horrible session. And my view is like, once you get to like, okay, I'm micromanaging a bunch of them, the next phase is looking at those as, like, cool. Everything inside that box is a run. And I am working with many runs, not micromanaging a single run. And I know I'm going way tracking, so. Far.
Them: I think, I think the question high level, I totally get it, makes a lot of sense. So the. The question for me is more like, how do you do this in practice? Like when you have Claude code, for example? I'm not sure. Maybe I missed something. But as far as I can see, there's no like Auto. Except for the whole time. So it's, like, really annoying from my perspective. It was. The only thing that I do is, like, always say yes anyway, because, like, in the end, Like, let's be very real. So, like, I think one thing that changed much, I think in the old sense, they're going to have employees, humans. Doing the coding, then you optimize for minimal changes, and you optimize on the right architecture because the cost of changing is so high. Now, since the cost of changing is cheap. The question is, like, why do I optimize in the first place? So why I'm not. I'm not why I'm not building the thing that I want, and then things fall apart and then I refactor if it's more difficult if you have customers on. On that product. But even then.
Me: Right. Exactly.
Them: The changes manageable. Like, if you're not Googling, you don't have, like, millions of, like, things there, but.
Me: Especially if you're able to be there thinking about that higher level of what's important.
Them: Yep.
Me: From the customer product perspective.
Them: So? So? So that's why. That's why. I'm carrying. Yes. I care about roughly the architecture, so I typically give it, like, very rough, very narrow instructions. I say, hey.
Me: Got. It.
Them: Definitely next. JS Definitely tailwind. Like, this is like the. The. Essentially The. The boundaries that you have and, like, everything else, like, make it work. And so the. That works pretty well. But again, I think what I'd like to get to is where. I, for example, what I've done is I get. We have this. White node JS app and I converted this over to next js. So this essentially is very straightforward. The easy thing is. Take this. Convert this to Next JS and make sure all end to end has passed like. This would be essentially what I'm saying. I want exactly the same app just like with Andrews. So the problem is this is a very long running task with like and essentially I needed to micromanagement claw to like get on the right idea because, like, then it fucked up, like, a lot of things.
Me: Yeah. Y.
Them: And then I need to, like, revert this, and then I need to come back and say, okay, I'll do it this way. And then I essentially needed to, like, have enough stuff so that it could run the end to end test. Only then to recover that, it needs to, like, convert the. Which then figures out, okay. I should have, like, used the IDs for the components, which makes sense. It should have done this beforehand. But anyway, I think that that's. A project of this size requires, like, something above cloud code, which is. And that's what I'm trying to figure out, like, how you do that.
Me: Es. Okay? Let's talk about multiple things here. So things like Ralph we need to chat about. I don't know if you've heard of Ralph.
Them: No.
Me: Sandboxing verification. The physics of the system. And so maybe let's start with a physics of the system. Which is. Or in other ways. You pointed out architecture. Like rewriting things. Actually, you have to push Claude and these coding agents to say, no, please be surgical, because it wants to just change everything. It's like, I don't care. Let's switch to Haskell. They don't care anything. But that means. Though, that we could do things that if you gave like a human to say. Give me 100 variations on this login page. They would be like, I am looking for a new job. They would never do that. But it's like, we could build new software development, lifecycle processes. And so it comes down to then, I think, what is the verification flow? And so one mental model is that start treating these runs or meta runs as the same way, maybe as like, if you had infinite number of contractors or infinite number of teams that you could sub out to and they were all competing with each other. In that world, though, especially with contractors or outsourcing things like that Unit testing or integration tests. And this may not be as applicable to you, but this is just how I think about it. If they own the code, they own the unit tests. Because they need to evolve those. And so me trusting the unit test to say, like, yeah, I should merge that is silly, because they could make it say, return true, or they could be testing the wrong thing. And so I think of it like automated QA or verification flows or smoke testing. You need that sort of equivalence. Some of those can be fuzzy still, because if you prompt the agents. And have tools to let them browse visually, they actually can do a pretty good job of being critical if you prompt them correctly, because they try to tell you that this is the greatest thing since sliced bread, because they like to be sycophantic. But if you're like, this was implemented by a candidate and we want to make sure we hire the best people. It can be critical then. That's one example of it. First of all, it's like, yes, you need to start thinking about what is the verification flow. And then me as the human that's in charge of this, that's what we really own, which is, what are the flows? How do we make sure that we verify them? And I spend more of my time managing that. I still will leverage QuietCode to help implement that, but it's back in the days of testing, just with unit tests. It's like make sure your tests are readable. Dry isn't actually a goal. With testing, you want to make sure that each one is a flow in testing one thing. And so I think about verification that same way. And you probably know playwright or selenium or something from back in the day. They're brittle, right, because you change the DOM tag, the test fucked. Whereas and if you read them, you won't necessarily get a sense of what is important and why. And so there's a tool called Stage Hand by browser base. You could use the two together, but all it's really doing is the idea that LLMs are actually good at taking. Like, hey, go log in. And then what you could do is it can look at the page DOM, and look at it and go like, okay, let's click edit and take a screenshot each time. And so there's tons of people who've taken tools. I'm sharing this specifically because you were mentioning a web page. When I think about verification. Which is, again, what I own. It's how do I make it so I'm just verifying the stuff where it actually takes my judgment when it's like, look, this page is ugly. It's all off. The login button is one letter at a time. There's no reason it should have had to get to my eyeballs. To give that sort of feedback. There are tools like stagehand and the idea of stagehand that you can run as ways to help do this verification flow. And actually before keep. Okay.
Them: Pe.
Me: Just from a so that's more on the how I think about it. And some of the tools Claude Code itself as well as Codex do offer some flags that let you say just like YOLO mode or dangerous mode, and the incantations that you have to do on them. Are annoying because like with quadco, there's dangerously allow all. Which means that you never click yes. It's just doing everything. Network request, rm, rf, everything. It's just like, cool. Just do it. And so they actually make you also set an environmental flag called is sandbox equals one. In order to really mean you're in a sandbox. You can do whatever you want, and if you fuck up, it's your problem. And so that's why it's like I actually run all of these things when I'm actually doing automation inside of their own container or vm. And so that way you can make sure that you set up the environment such that maybe they have git poll access, maybe they even have the ability to send to specific branches, or maybe you limit their blast radius because it's like they're not going to be able to RMRF your hard drive.
Them: Yep. Yep, that makes sense.
Me: And so that's just a pragmatic thing. Now, all of these tools also have what are called SDKs. And it's really confusing because I think in Claude's case, it's called, like, the Claude Agent SDK maybe. And Codex, I think, is just called the Codex SDK and is only written for Typescript or something else. Even though Codex is implemented in Rust. And then gemini. They have something equivalent. I use Gemini for very specific things, and I usually don't have to automate it. Complicated. I could actually just send in a single prompt and I don't have to do the automation just from a pragmatic put it into sandbox. That could be a VM vagrant even. Like if you want to go way back or whatever sort of system you want and then just kick it off with a sending in the prompt and saying YOLO mode. Now. Sometimes. Actually, most of the time, I don't find that to be enough.
Them: Okay?
Me: And that's where the concept of Ralph comes in. And so it's Ralph Wiggums from the Simpsons, which is the character that is really stupid as, like, mifale English that impossible. And so this guy came up with the realized that, hey, just put Claude code in a loop. Where you have it, do the same thing over and over until it's done. Is way more effective than you would have thought it would be. So while true, while not complete. Claude code fix. This is roughly what Ralph is. Now, this is one of those examples where. Conceptually, there's a lot of interesting things there, especially if you combine it with the idea that verification and all this other stuff, and start thinking about it, about the software development lifecycle. You could read his stuff or read people talking about Ralph or the controversies around it. But the thing to note is that with the SDKs whether it be amp Claude. Codexes. It's literally like a 1015 line piece of code that is trying to say, here's how you do it. There's some best practices when it comes to using these coding agents, which is try to only have them do one thing. One thing if you haven't tried to do too much in a loop. It just won't work because the context window gets too long. It's trying to think about things that don't matter. And so the RALPH loop that we were using about three or four months ago was, while true, while uncompleted task in the EPIC that it's supposed to be working on. Create a new instance of Claude. And the Claude task was. The prompt was our specification document. Find an incomplete task and the next incomplete task that makes sense to work upon, work on it. And then because we were using, in this case, we were using either the Claude SDK or the app SDK at the end. That's just like a four line loop of saying while there's messages, run it. And then we would print out each sort of just like the first 40, 80 characters of each sort. Of message from the it. So I'm not watching what it's doing at all. I'm just kind of maybe seeing what those things are. But as I started going, why only run one? Why run 10 of them? Because if one messes up, I don't care. Throw away that sandbox, start over, because the probability of it gets stuck at twice in the same way. So treating these things as more of, like, experiments that I want to remind many of them rather than just a single one. So again, while not complete, Say create a new session. That is like, work on this. And make progress, and then I do two things afterwards. And actually it evolved to three. Originally it was just two, which was if the git tree is not clean, send a message back to it saying, hey, your git tree is not clean. Are you sure you're at a next stopping point? And basically give it three tries, like going back to the top and coming out. And then next is automatically running the lint and the unit tests and the integration tests and all of that fun stuff. And again, that doesn't mean just because they're passing doesn't mean that it's going to be. What I want, but it's better than. It's kind of like trying to help push it into the right direction if the unit tests aren't passing because they get to this point where it's like, well, it's broken, but it's not my fault. It's like, no, this is. Totally your fucking fault. Do it. Fix it.
Them: Yeah. So those things we are actually putting in the Claude skills and, like, Claude MD Files. So Claude essentially is pretty good and, like, passing those tests. Once. It's like knowing this, but it makes sense. So essentially what it described is what I. The raw loop is what I have done without Roth. I would say for the ui,
Me: Yeah.
Them: It makes. It makes a lot of sense. Specifically, I did exactly what you said. Make an implementation plan, analyze the code base and then like do it per biggest feature and then iterate over each feature and then complete this and like once and do it by criticality. So that I actually made a sense of urgency by saying I have to ship this on Monday, so, like, do this now.
Me: Exactly.
Them: So that that has worked pretty well. But yeah. So because, like, the agents tend to be. Lazy. I would say so. Like, they're so. You always need to, like, force them to reevaluate what they do. So, in my cloud code, as soon as it goes a little bit beyond certain scope, Then it's always like trying to shortcut and leave things out.
Me: If it gets above 50% context window, Claude just goes like, I need to just get done any way possible, including just lying.
Them: And. So. So that's why I think, like, what I recently used most, the plan mode, to be honest, like, that works relatively. I need to see, like, how RAV works with plan mode and combination, but, like, that was probably working. Because I. I typically now tend to complex stuff. Put it in plan mode and then it has the to do list and then it has like smaller context per execution of task. That. That seems to work pretty well. So. So. Makes a lot of sense, I think. Sandboxing is a good one. So I think also for the. Browser. I just can't. It's pretty nice. I think I'm going to, like, flip this over to this one. So that. Actually the thing that once I have solid station, it looks like I should actually do this for mql, where people can write policies in human text, and that's just doing it.
Me: Yeah, well, and that's the overall thing is, like, I didn't. Like, that's what we were doing three or four months ago. And at the time, Claude Coe didn't do any of this sort of. It now actually has a little plugin called Ralph. And also, I don't know if you noticed, in the last week Claude started doing this thing where what it's done with the plan, it will be like, let's clear the context and start.
Them: Yep. Yeah.
Me: They're realizing that, and it's because the idea that you keep on using the same context just doesn't work. Actually, what's interesting is Codex. You could actually tell it to do something really complicated and its compaction actually works. And it could run for. I've had something run for four hours and implement something complicated again, backend these stuff. I do. Or system or architecture stuff. I boot it over there.
Them: Yep.
Me: But I guess. What the tools do. And this is actually why our company is pivoting. What they do in the agentic coding systems versus what the models do versus what you need to wrap around them or constantly changing. That's why what we really need to be thinking about always is what is the proper physics, stlc, et cetera. Up until two weeks ago or even maybe a week ago, I would have said you should be doing that sort of meta checking outside of the context. Because outside of telling it to check git can be a shell script. That's an if statement, not a use fuzziness and probabilistic nature and what LLMs bring when you need it. And to say you must always have a clean work tree and you must always do the tests must pass. And then running your verification steps to say you could only move forward if those pass and that to be acceptance criteria. So that's what we've added is we will now have, for instance, on one of our systems I'm working on right now. A 25 minute verification loop. Vacation step where it's like deploying to GCP and VMs and K's and all of this like a complete new cluster to verify that the flows that are important actually 100% work. And then we provide that feedback. It can make changes, but then it has to. Again, my goal isn't that at the end, I have the golden pr, the golden poll request. What I want is that I have the ability to. I am more an expert at the end of how our system works. Because if it's just generating PRs and they're just getting better and better, but you're just a monkey pulling the lever of saying, well, this. What value are we adding other than clicking the yes button over and over? So trying to think about, like, okay,
Them: Yeah.
Me: Adding in a couple of the interesting things that we've added in for the last two months is what a PR has done. When the model thinks it's done, sending it to Gemini in this case, because Gemini, with this huge context, is much better at Do a systematic review of this review is meant to be write a document of what has changed. Why? For people who have been building this because you're doing this and people don't understand this PR you. It's been worked on for a while, et cetera. That sort of prompting. I forget our exact wordage. It's changed over the last few months. But the goal is critical review, that is informing me or others of how it's changed so that they could still understand how the system works. And then similarly, what we've added is if verification, things have changed, you need to provide justification of why. Explain the verification steps were wrong or this is important because that's then something I 100% pay attention to. And then another thing that's been really interesting is taking five different implementations of the same thing, and this is more for backendy, and then having it write the equivalent of a blog post of somebody who would have been writing this. So it feels more like me as a developer back when it would be you could actually search the Internet for how to use some technology. It just happens to be the perfect blog post for you. But then you could scroll through it really quickly and go like, okay, that's right. No, that's wrong. And then that sort of 80%, yeah.
Them: I think that's dramatically changing. Because I exactly what you say. Two things that I want to say to this, like, for UI development. I think it's pretty annoying that I don't. I'm not able to. To describe the component. I cannot just annotate the component, and then if I just, like, do it in that context of the thing that I touched. I think that will change. I think the other thing I would 100% agree with you, like. I would like go the opposite. I think it's. I would say engineering has not developed for the last 10 years. Before. Like, let's say before cloud code, like was in a good shape. Like end of last year. Engineering was for the last 10 years the same, with better SDKs, better CPUs, faster. But conceptually nothing has changed, so no better abstraction. So. So we essentially struggled from like we spent so much more time from high level idea to getting to the first iteration and implementation, whereas now you essentially move the mental cycle for us like More on what's the problem, what we solve. And not on how. Right. So, like, the. The shift is. Dramatically because the cycles are quick. So essentially, you could argue it was wasted of time what we have done all the time, I mean.
Me: Yeah.
Them: Historically, like you're typewriters in the past was so. So it makes sense that we have done it. But like the. But I think there's no immediate value in, like, me changing a Kubernetes operator.
Me: Yeah.
Them: By the outcome should be. I have a Kubernetes operator that works as I want to have it.
Me: Y.
Them: And if it like this way or that way, SDK decides it anyway. So I just have to follow the specification no matter what. So why do I need to expend all the time?
Me: Eah.
Them: My value. So the value is the product that you're offering, which offers something on top. The technology is just the mean to get there. So then in this sense, I think exactly what you described like now, Now I think we, since we have more mental capacity.
Me: Right.
Them: Because we don't have the coding now. The question is, like, I think the iteration of system development is, is getting much quicker now because we have more mental capacity to work on this.
Me: Right. Right.
Them: Before. I've seen this with marketing. Like, marketing for me is like very eye open. Because we worked with an agency. So what happened is they wanted to change the website. So they described to the agency that they should do something and the agency did something. And then it was not the right thing. So it was slow prompt and getting this implemented. Right. So now you have instant feedback and the website is not great. Cool. Like, let's get it out. You can change it tomorrow. It's no problem. You can iterate like multiple times.
Me: Right. Yep.
Them: And now the. The. People. You know what we want. Or sometimes we don't, but we can try. But the. Before, the iteration cycles were so slow.
Me: Right. Right.
Them: That people essentially, like, were stuck in, like, the execution.
Me: Yeah.
Them: And I think this where we get out, so. Exactly what you say. I think it's. It's interesting. How is engineering? We come in like when? You don't. Don't do the low level stuff. It feels like you're going from assembler to the higher level language, and now we go to another higher level language.
Me: Right. Exactly. Yeah. And it's like, I think there's multiple ways this can go and which. Is is it that we as engineers will become more like principal engineers that are working with tls and lots of different things, or is it that everybody's becoming a product person, that we will seed all technical ground to them? And right now it feels like there's still a lot of value in being able to your hard won experience over 20 years. You're not the one implementing it now. But like a principal engineer is. Like, we should be doing it this way. I know that this is not actually the way we want to do it. But you're not going to go in there and fix it because, like, you'll just send it back to the team. And so similarly like having that taste or insight. And right now it feels like there's maybe on a website it's less true because you're like, I just want the website to fucking work. I don't really care. You use React or Vanilla js. Don't care. Just make it work.
Them: Yep. Website is different because, like, you can change this, but I think if you have a critical system running, it's definitely more complicated, and I think. My take would be you will need more engineers that are more expensive. Because the complexity is increasing, you need to manage more. And I think engineers are already struggling with the technical complexity. So if we now just like say, let's say I don't need a kubernetes specialist anymore to manage kubernetes. Okay, so then ok, but if a company is by. By our product, they don't care. They say mondu. That you're responsible, so you fix this stuff. So you still need, I think a person that in the end is like, making sure this stuff works as it should be, even though the person doesn't implement it. And. And that's why I see even with Clover be the best encoding. People will still go to an agency and say, like, do it for me, because they don't want to maintain that stuff. Because, like, I had the discussion with a friend. Exactly. That I said, like, hey, why have you outsourced this? They can do it in five minutes for me. They said yes, but then I have to spend time on this, which I don't want to, so I pay for it so that I don't have. To do that. So I think the one is the implementation side. But, like, who's responsible for what? You essentially outsource the responsibility, not the implementation. So I think that will still happen. Let's see. I think a lot of, like, junior engineers still have a problem.
Me: Oh, yeah, yeah. I think having the experience to know which way. Is, like, the right way. Is important. But if we are saying that. Okay, cool. We could. Do we need less or more? Do we need, like, it's all still up in the air and it's changing so fast? But it's like, how are the juniors going to get that experience if, like, if we're not. If there's no hiring pipeline anymore? For people going to have a junior job. That's a bigger question.
Them: But the question is, maybe nobody needs to know this anymore because, like, nobody needs to know the assembler anyhow. So you have five people, probably at Microsoft and like Apple, that actually know assembler inside out for their operating system. Of a little bit more. But, like, then. Except from that, like, nobody is doing that, right? So the. I think probably in the future probably don't need to have that knowledge anymore because the extractions are just getting better. So then if the testing comes with it, like, then who cares?
Me: We should talk again. And I'll grab a beer while we're having that conversation.
Them: Yes. Thank you for the links. That's.
Me: This is augmentation. I've seen other ones like it, but this is the one that just showed up a day or two ago. And I've seen lots of different people do this in different ways, where you have a way to basically provide that grounding and insight back to it. Another one that I've really liked is the idea that you embed a little. You have a little thing that will just record people's voices and what page they're on and just, like, have people walk through all of the pages and, like, provide live feedback of what? Like, oh, that's surprising. Kind of like. I guess this is why, again, I think of everything about, like, product design or. Like, software design life cycles. It's like, okay, how do you get feedback? Okay, how does that change? Okay, can we actually leverage agentic feedback systems? Yes, you can. And maybe it gets you through. Get you from other shit to, like. Okay, it's worth actually looking at. And then maybe there's a really quick, like, Tinder interface that you go swipe left and right from. Like, just first feelings of whether you like it or not. If you have these bots generating it. And then at the end. Like the way that we are gathering feedback from. There's so much in human psychology of like, you know, group think and all of these different pitfalls we have as humans about getting feedback and sharing it, that if you build like bot agent as mediator or make it so everybody could provide their feedback and it synthesizes. It without having it be like, oh, the first person who talked in a meeting is setting the context of everybody else is responding to that or thinking about that. So it is in my mind, like you said. It's like, we haven't moved in 10 years. It is like, okay, what is this higher level of what actual product development and software development could be like, when the laws of physics have changed, where an implementation can be redone 15 times in a day or over? Lunch. And that is something now that like the fact that we have little Kanban cards where we could by just putting a card into the research column, it will click off a Claude code that will go then do analysis based upon what's on that card and come back with a report. And I can actually just click I want five different reports right there. Another thing I'm doing is like if there's an important project and actually just ran this on your all's thing is have a Claude code session where I say hey. For every commit in the last week. Create a sub agent. And this is me having to know, like how Claude works. Create a sub agent that's just analyzes from a user's point of view what this commit is all about. And this was a very broad one because I'm not a Mandu user. So I was like, okay, just from a user customer experience. Point of view, analyze this commit. And then, so now I had a markdown file for each of those commits. And so you could keep that sort of thing then for each of these commits. Sorry. For each day, analyze, summarize the summaries. And for each week, summarize the summaries. And you come up with these sort of, like, documents. So when I think about the agents creating more and more code, it's like, okay, how do we deal with that? It's not going to be keeping up with, reviewing every single line. It's coming up with what are the systems and processes that actually help us? Just like if you were a principal engineer and you had 50. 15 teams that you're working with, and you know there's 100 people on the, you're not going to be able to keep up with the code review. But you can keep up with. In the old days, it'd be like talking with EMTL, PM, etc. On how things are working and why and what is the priorities and how. Like making sure each of them doesn't end up in a local maxima. Because that's the problem in my mind is all teams. Everybody ends up in local maximize and so the more you could move to a higher level.
Them: I think this will be dramatically a big problem for larger enterprises. The reason is because everybody who's not able to code has now claude code. And so they. They tend to. Do some codings for this over the fence and hope it's, like, being attainable. So then. And this will mean, like, people will put stuff online, and then the question is, what? So the. So now, from security point of view, is like, one thing. Like, now. Yeah. Like 20,000 small things that are not connected. Nobody knows about it, Nobody knows how to manage it. Like, it's like becoming a nightmare. The other thing is, like, even if you say it's not a security risk, then the question is still, like, who understands how this is all connecting? And I mean, like, Even right now, large companies have problems like understanding how things connect to each other. When I started at dt, like, this super huge, like, war, where, like, the whole system was designed. Problem was that those people who drew those, like, architectural diagrams that never touched a real system, so there was not even accurate.
Me: Right.
Them: But like, the. So the interesting thing is that I think the complexity of like, those systems where we come very interesting. And like, managing that, I think there is something that's similar to what we have with L and M. Like, we can't explain the system anymore, so very soon, and we need to figure out, like, how to. Make that complexity manageable again.
Me: Yeah.
Them: So I think that's really tricky. That will be tricky.
Me: But I think also it gives us new tools again where it's like before you could. If it was an important change, you're like, why the hell is this happening? You could drill down to actually talking to the team that did it, the person who did it, and then asked detailed question, asked research questions, asked like, what if it was this other way? That becomes something that you can't do the same thing. But just saying again, like just saying, Claude's job is to create the golden PRs, I think is us, like, removing our responsibility from both product engineering and product development. It's like, okay. If these are making changes to systems. That are modifying the contracts between it or the interfaces or what have you. How are we, you know, the fact that we could kick off a Deep Research or the equivalent of, like, analysis or run 100 of them. There was a podcast by the Deep Research team at the beginning of 2025. And this was the 01 Pro sort of team, and it was like the first thinking model that came out. And the A16Z, I think, person was asking, like, what do you do inside of OpenAI that you don't see people doing outside of open API OpenAI with models. And that really surprises you. And they're like, everybody outside. Every time anybody's like, oh, check all this cool thing, they're just one shotting it. They're just asking it to do it once. And they're like, if this is important, Like these are probabilistic machines, and if they fall down a hole, You're going to get one answer. Because they were trained to simulate the conspiracy theorists, the bad code, the good code, everything. And so if the probabilistic thing puts it down a wrong path, You're. And what's interesting is, like, these, these agentic systems and these ralphing loops and these verification loops try to push it in the right direction. But again, like, if it makes a wrong decision from an architecture point of view, you could be stuck in that. And so rather, like, can you push it in? Multiple directions. Can you have this sort of like, oh, every time there's a PR kick off 10 critical reviews in different ways before a human even looks at it. And then what you're really doing is sort of like understanding the theory of the system and how it's evolved.
Them: I think. I think it switches to. To the intent. So. So the question is if you think I'm like, if I would, like, start from scratch, probably, like, so it doesn't even matter if it's a live system. So to avoid copyright in this, like, the first thing you would say is, like, okay.
Me: Yes.
Them: Take this. Go code convert this to rust. Done right. So that means you can like so you very soon. You cannot differentiate on the feature to be built because others can build this in a second as well.
Me: Yep.
Them: So then the. The question is, is not the feature per se, but, like, building it together in a way that people see value? So the. And I think businesses have this already from my perspective. I mean, that's why we see business exist. Because they didn't know which works and then so they build the probability run like multiple startups and prolo let's see what sticks and make it some not essentially the same thing just as a slower scale. So now with AI you can shortcut this more but I then the the interesting thing is you can now iterate on the business problem faster and so less before I took time like people needed to have the right intuition. But I think even with AI, you can still.
Me: Yes.
Them: Spend like. Hundred thousands in the wrong direction, so. Because, like, it's doing the thing that you're saying. Since from my perspective has no intention like it doesn't. Know what you want to build. Like it's just something that you say and may fit.
Me: Yeah.
Them: So the. So I think that will be interesting because we can probably. Overall, not just as individual companies, but overall, I think we can. Get into an area where you can, like, Develop newer things faster because you're not stuck in basics.
Me: Yeah.
Them: Just thinking about, like, building a website next. JS is the best example where you. It doesn't matter if you, like, do the CSS design perfectly on the button on the website. That's been done, like, 20 million times, so there's no value in you to. Same thing again.
Me: Yeah.
Them: What is, is a problem is like building something where you connect the right things, where it's a user problem. And I think that's still, that's still the problem, only that you iterate faster.
Me: Yeah. So are you seeing the chats in the sidebar, by the way? Okay?
Them: Already is for read already on my list.
Me: So, Simon Willison, if you don't read, like, if you want to keep up on what's going on, AI just read his blog. And, like, you don't have to, like. There's enough coming out where it is a sort of commitment of an hour or two a week maybe. But then you don't have to follow around with everything going out there because Simon is one of the most connected and like, has been blogging this sort of thing. Period. And so, like, this was an example of the fact that Codex, plus a bunch of verifications ran for 4.5 hours. And made it such that he was able to port a library from Python to JavaScript. And, you know, it was free because he just used his Codex CLI, $200 a month subscription. But by having these verifications, steps, he was able to, you know, to just let it do its thing and it knew when it was succeeding or not. And so. But this is also where he only used one. And in my view, like, it could have been where. It's like it made the decision on architecture that made it over Leverbose or overly whatever. It's like maybe performance matters or maybe verbosity matters. It's interesting that a lot of code I see the LLMs are preferring. Not like having more flatness than lots of depth and recursion and things like that. Not recurring. Lots of, like, sub calls and lots of like, oh, they're okay with repeating yourself. But it's like, okay, well, then you need things that help make sure that you're not having subtle bugs. Because, like, this flow is different than this flow and things, but should be the same.
Them: Y. Eah, I. I've seen this too. So that's definitely a problem. Like it tends to over like re engineer the same thing that it already has to be a lot like it's that are like abstracting it.
Me: So there's a code analysis tool for. I Forget if it's JavaScript or Rust, but it basically looks for similarity, but not at the, like, more at the ast conceptual level than at the, you know, character for character level. And then, like, those are the sort of, like, theoretic tools that. Somebody said recently that I was like, oh, yeah, this is absolutely true. There's all these great ideas. Programming languages haven't really evolved. Since. The 80s, 70s, maybe whatever year you want to choose. But it's like very slow because you have this chicken and egg. It's like you have to have enough adoption for people to actually. And it's like, how are you going to get enough adoption to make it worth putting the time into it? And it's like you end up. So there's all these ideas around how we could have more. You know, formality or provable or whatever you want to, you know. And in the past, it's been like we haven't had improvements in tools because of that. And I think that that is. That's no longer going to be true because. I have switched to Rust. Like the amount of Rust training data out there is significantly smaller than JavaScript or almost any other language. But it's gotten enough. And the fact that RL works as well as it does. You don't have to keep up with this. But it's like the paper last year. The Chinese team Deepseek. Like, everybody was like, oh, like, like what they really did. That was crazy. Which actually came out in the fall of the previous year of 2024. So it was already known is they figure it out that you could do a RL flow to reinforce most of the training of models now. Happen after you're done with normal training with reinforcement learning to try to push it in the right direction and not do that. You're on the stupid conspiracy theorist bug ridding code track. You're in the smart track. But that means that although the patterns were like, you need ratings on every single one to say, like, is it, you know, giving the right score? And what Deepsea figured out was a very efficient way of training where you just have to be able to say which of two results are better. You just had to have comparative of two answers and then you could do fine tuning significantly faster with RL and being able to say this is better than that. It's much easier than an absolute ranking of thousands of choices. And so all of that, to me says, we're going to see an explosion. In programming languages designed to be agentic and LLM friendly from the beginning.
Them: Yep. Let's see. Let's see how this. Shapes ultimate, isn't there that there was another programming language coming out that is just like completely written by I. And I think the. The interesting thing is that. You can now have a few agents, like, writing a full, full fledged language, right? Like, so much faster, including ecosystem stuff. And what is also interesting, you can. Analyze all the shortcomings from existing ones and you can actually make a better one. Because you can like if you, if you like. Typically, the languages have very well documented, like shortcomings. And so finding a better balance between those shortcomings is probably possible. So we will see. You will see.
Me: Yeah. One last thing and if you want to talk again later. I love this, but you had said you try to keep it as concise as possible for Claude code. What I found is that there are times where that's the right strategy, but then there are times where it's like you do not want to constrain. It. What you want to do is tell it like, this is the direction I want to go. But what you want to do is make sure that it doesn't spend. Like in Codex's case, it could spend hours. So you don't want to spend two hours and maybe five of them going concurrently. And it's in the wrong direction.
Them: 100%.
Me: So what I start with is actually a different model than anything we've talked about, which is. Codex OpenAI has 5.2 of chatgpt. 5.2 extra high thinking. And they have the fine tune 5.2, which is called 5.2 Codex. So when I'm doing dev. When I'm actually having an implement. Codex 5.2 high, not extra high. Extra high is almost never worth it because it's just so much longer and so much more use of your budget. But 5.2 high of codecs is great for coding. But then when I want it to think and understand, like do we have a well designed spec? What are the open questions based upon what I said that you want answered? So that way. So you prompt it to be like, here's what I want. Go look at this stuff. Look at the repos or look at whatever. And what isn't clear about my ask that I could answer for you before we go. And then I'll do that and then I'll switch to I'll either then have it write out a markdown that I then send to 5.2 codecs. Or I'll just switch the models back to 5.2 codecs. Either through automation or through manually doing it. If it's like, really something I'm paying more attention to to actually do the implementation. But the point is, like, that was mixing both how I do it and what I do. It's like, try to just tell the model, like, what is confusing here, like, because actually Claude is okay with uncertainty. Whereas if you tell Codex, A and B and A and B are separate, independent of each other. Sorry. No, sorry. Contradict each other.
Them: Yeah. Yeah. That makes a lot of sense.
Me: Codex goes fucking crazy. So you have to be very careful because it's like codex inconsistencies and things like that. It will just try to, like, get to a solution that solves both of them and not be able to. Whereas Claude is happy to just go, like, whatever happens, happens. I'm just going. To go with laissez faire.
Them: Yeah, you see that it's optimized for coding. So, like, then. So the thing that is. I mean, like, all those models are always biased, so you need to, like, be very careful with the prompts. So if I know what I want, like, I'm very specific because, like, then I say, use this, use this API and use this, this, this, this. So then it's executing it, right? But if it's. Sometimes I want to, If I want to evaluate code, like, I'm, I'm very specifically using. Very vague, like, open questions, because otherwise it's, like, biased to what I've wrote. So. So I'm leaving. What you just said earlier.
Me: Yeah. Right.
Them: I explicitly leave words out. Because then it's like, not, like, biasing in one direction, so that makes a lot of sense.
Me: And what's fun is you could actually even again. Like this is why I think it's good that Kodak and Claude is adding more and more of these capabilities. But I still think that their agentic loop is they're trying to sell for everything for everybody. And skills and whatnot makes it pluggable, but you need to be able to control this. So it's like, okay, we have a flow where it's like, it takes a very defined spec. And then it will create five variations, some of which have a rule. Like, remove some specificness, make it more general. Like, there's a whole world of genetic algorithms where you try to get out of the mode and you try to actually explore variations. Because it's like, sometimes you'll find those kernels of nuggets of like, yes, that's a great idea. Let's pluck that out. And I don't care about the end results. It's, like, cool. This concept is great. Now let's write it there.
Them: Yep. I think. Yes. Yes.
Me: Cool.
Them: Yeah, let's definitely do another round on this. I will definitely try out all the cool stuff. I've seen the Ralph stuff but haven't used it, so, like, that's definitely a good reminder.
Me: I know you. I would not use. I would not go find somebody's ralph. I would just say the thing to take from it is looping with verification.
Them: So.
Me: And you could write your own, Ralph. In 10 minutes.
Them: I think definitely I'm trying this agentation, and, you know, that seems pretty cool. But I think will help me specifically pinpointing issues with my port. So, like that we'll definitely try this, so that's cool, I think. On the testing front. This was definitely also super helpful because I will test stage. And. I think the idea, like, Writing policies in this way. I think I've used it in a pre compiled stage, like where you essentially have. You write a policy, and as soon as you upload this or some even locally, you can convert this into, like, something, and it just converts it into approvable mechanism. But like, you don't have to do this. It has always a precompiler phase, right? Like so. And then you can recompile and then it stays the way. But like then you have the advantage of. If Mondu ever involves with MQL like this. You don't have to rewrite things. You just like, run the same policy. And that's. That's why I find this pretty cool, so. I will probably try to port this stage and thingy to us. So, let's see.
Me: Awesome.
Them: So cool stuff. Yeah.
Me: Yeah.
Them: Super helpful, Jesse. Thank you for your time.
Me: Yeah. Let me know when you want to chat again. Like, it could be, like, every week for a while, because, like, this stuff is, like, talking about how it actually works. For people. And, like, this is how we figure out the best practices. And so everybody's finding different things every day.
Them: I'm not sure if I can do every week, to be honest, but, like, let's. Let's maybe add for every second week or so.
Me: Y. Eah. Cool. Awesome. Sounds great. Have a good one.
Them: Cool, jesse.

</details>
