# How Large Language Models Affect the Web Ecosystem
## Deep Research Report — July 2026

---

## Executive Summary

Large language models (LLMs) are reshaping the web at a fundamental level across four interconnected dimensions: scraping the web for training data, flooding it with generated content, consuming its traffic through AI-powered search, and overwhelming it with automated bot traffic. The cumulative effect is a web that is simultaneously being extracted from, polluted, defunded, and overrun — creating what some researchers call an existential crisis for the open internet.

---

## 1. Scraping the Web for Training Data

### The Scale of Extraction

LLMs from OpenAI, Google, Anthropic, Meta, and others were trained on vast swaths of the public web — billions of web pages, books, articles, images, and forum posts — largely without permission, compensation, or even notification to content creators. The Common Crawl dataset alone contains hundreds of billions of web pages and has been a primary data source for most major models.

### The Legal Landscape in 2026

The legal framework is rapidly evolving, with several landmark cases setting precedents:

**Thomson Reuters v. Ross Intelligence (D. Del., Feb 2025)** — The first final judgment on AI training data copyright. The court found that using Westlaw headnotes to train a legal AI was **not fair use**, focusing on market harm — the AI competed directly with Westlaw's research services. This ruling favors rights holders.

**NYT v. OpenAI (S.D.N.Y.)** — The bellwether case. The court denied OpenAI's motion to dismiss, rejecting the argument that AI training is inherently transformative. Key ruling: when AI outputs compete with the original content's market (e.g., ChatGPT generating news summaries that substitute for newspaper subscriptions), fair use arguments fail. Discovery phase as of April 2026; trial expected late 2026 or early 2027.

**Getty Images v. Stability AI (D. Del.)** — Stable Diffusion was trained on 12 million Getty images without license. Damning evidence: AI outputs that include garbled Getty watermarks, proving direct copying. Trial expected 2026.

**Bartz v. Meta (N.D. Cal.)** — Class action over Meta training Llama on the Books3 dataset (pirated books from Library Genesis). The case targets clearly pirated content — the weakest fair use position. Class certified in 2025; proceeding on damages, not liability.

**Concord Music v. Anthropic (M.D. Tenn.)** — Tests whether an AI's ability to reproduce copyrighted song lyrics on demand constitutes infringement, even if training is arguably fair use.

### The Emerging Legal Framework

Courts are converging on a **fact-specific market-harm test**, not a blanket rule:

1. **Training is not inherently transformative** — The Supreme Court's *Andy Warhol v. Goldsmith* (2023) narrowed the transformative use doctrine
2. **Market effect is the decisive factor** — When AI outputs compete with training data sources, courts find infringement
3. **Piracy is a bright line** — Using pirated/unlicensed content creates nearly indefensible liability
4. **Licensing is the direction of travel** — OpenAI (AP, Axel Springer), Google (Reddit, Stack Overflow, news publishers) have pivoted to licensing deals

### The International Divergence

| Jurisdiction | Approach |
|---|---|
| **US** | Case-by-case fair use analysis; no specific AI training exception |
| **EU** | Text/data mining permitted for research; commercial use requires rights-holder opt-out via robots.txt/TDM headers |
| **UK** | No specific AI training exception; narrow fair dealing doctrine applies |
| **Japan** | Most AI-friendly — permits AI training on copyrighted works for "information analysis" without consent |

### The Technical Arms Race

**robots.txt is failing as a defense:**
- Cloudflare documented Perplexity using **stealth, undeclared crawlers** to evade robots.txt directives (August 2025)
- Perplexity rotated user agents (impersonating Chrome on macOS), cycled IP addresses and ASNs to bypass blocks
- Activity observed across **tens of thousands of domains and millions of requests per day**
- Cloudflare de-listed Perplexity as a verified bot and added blocking heuristics
- In contrast, OpenAI's ChatGPT Agent respects robots.txt and uses the Web Bot Auth standard

**The defense stack:**
- `robots.txt` directives (TDM Reservation headers)
- Cloudflare's "Block AI Scrapers" toggle (now default-on for new zones)
- WAF rules targeting specific crawlers
- Legal action (DMCA, copyright registration for statutory damages)

**Key quote** (Cloudflare blog): *"The Internet as we have known it for the past three decades is rapidly changing, but one thing remains constant: it is built on trust."*

---

## 2. Flooding the Web with AI-Generated Content ("AI Slop")

### The Scale of Contamination

**Graphite (2025)** analyzed 65,000 English-language articles and found that **over 50% of new articles on the internet are now AI-generated** — up from ~10% in late 2022. The ratio plateaued at roughly 50-50 by late 2024.

**"AI slop" was named Merriam-Webster's Word of the Year 2025** and Australia's Macquarie Dictionary Word of the Year.

**Meltwater** found mentions of "AI slop" across the internet increased **ninefold** from 2024 to 2025, with negative sentiment hitting 54% in October 2025.

### The Content Farm Economy

A Guardian investigation profiled an AI content creator in Ukraine who operated **930 YouTube channels** (270 monetized) with a team of 15 people, clearing up to $20,000/month. The content ranged from AI-generated music over "sexy AI girls" to life stories written by ChatGPT for elderly listeners.

Key quote: *"It was a conveyor belt, with fairly low quality... To make money here, you need to spend as little as possible. YouTube is basically just clickbait and sexualization."*

Only the **top 5%** of AI content creators ever monetize a video; **1%** make a living from it.

### The Philosophical Problem

Thai Vo-Nhu (American Philosophical Association blog) argues AI slop represents **"a crisis not of quality but of authenticity"** — drawing on Confucian philosophy:

> *"Large language models are the mechanization of the Village Worthy — someone who appears virtuous but lacks any genuine moral core. They produce the forms of virtue, creativity, empathy, or intimacy without any corresponding substance."*

Real-world harm: A funeral platform's AI wrote that a deceased person found "joy in the gentle keys of her piano" — except the deceased never owned or played a piano. The AI fabricated it because "grandmother" and "piano" are statistically adjacent in training data.

### AI Slop in Academia

The contamination is spreading to scientific literature:
- **21% of reviews** at the International Conference on Learning Representations (ICLR) in 2025 were fully AI-generated (Financial Times)
- AI-generated manuscripts are **harder to read, more jargon-laden, and more likely to be rejected** (Forbes, April 2026)
- AI-fabricated "junk science" is flooding Google Scholar (University of Borås study)
- arXiv has implemented bans on AI slop in research papers

### Content Quality Defense

Platforms are fighting back:
- YouTube has become more aggressive with takedowns of AI-generated content
- Google appears to be penalizing AI content in search results (86% of articles in Google Search results were human-written per Graphite)
- Meta's AI "Vibes" platform attracted only **23,000 daily active users** in Europe after launch — showing limited consumer appetite for pure AI content

---

## 3. Stealing Traffic Through AI-Powered Search

### The Zero-Click Apocalypse

Google AI Overviews and similar AI search features are devastating publisher traffic:

| Metric | Value | Source |
|---|---|---|
| US Google searches ending without a click | **58.5%** (640 of 1,000) | SparkToro/Datos, 2024 |
| Zero-click rate with AI Overview present | **83%** | Bain & Company/Dynata, Dec 2024 |
| Zero-click rate in Google AI Mode | **93%** | Semrush, Sep 2025 |
| Mobile zero-click rate | **77%** | SparkToro/Datos, 2024 |
| CTR drop on top-ranking page with AI Overview | **58%** | Ahrefs, Feb 2026 |
| CTR drop measured by Pew Research | **47% relative** (8% with AIO vs 15% without) | Pew Research, Jul 2025 (n=68,879) |
| Publisher referral traffic drop | **25%** | Digital Content Next |
| Search engine referrals at publishers | **33% fewer** | Chartbeat |
| Organic traffic decline across sectors | **15-25%** | Bain & Company, Feb 2025 |
| B2B websites with significant traffic loss | **73%** (avg 34% YoY decline) | Onely/ABM Agency, 2025 |
| Traditional search volume forecast drop by 2026 | **25%** | Gartner, Feb 2024 |

### The Great Decoupling

Publishers report a new pattern: **impressions stable or rising, clicks in free fall**. Google shows your content in AI Overviews (visibility), but users don't click through (no traffic). The content is consumed on Google's platform, not yours.

### Publisher Response

**Penske Media** (Rolling Stone, Variety, Hollywood Reporter) filed an antitrust lawsuit against Google.

The **European Publishers Council** filed a formal complaint with the EU Commission.

Google's defensive response (May 6, 2026): Five updates to AI Overviews including inline links, hover previews, "Subscribed" labels for news, and article suggestions at the end of AI answers. Widely interpreted as acknowledgment of the click crisis.

### The New Economics

- **80% of consumers** rely on AI-generated results for at least 40% of their searches (Bain)
- **85% of B2B buyers** form their vendor list before any search — making pre-search brand visibility critical
- Brands cited inside AI Overviews earn **35% more organic clicks** and **91% more paid clicks** than non-cited brands (Seer Interactive)
- AI search traffic growing **527% year-over-year** (Semrush); may surpass traditional search traffic by 2028

### Kellogg Insight's Strategic Framing

Northwestern Kellogg's analysis: *"Don't Panic — Evolve."* The advice is to diversify traffic sources, build direct audience relationships (email, subscriptions), and create content with unique value that AI can't replicate (original research, expert opinion, community).

---

## 4. AI Traffic Bots Overwhelming the Web

### The Inflection Point

**For the first time in history, bots outnumber humans on the internet.**

**Imperva 2025 Bad Bot Report:** Automated traffic surpassed human activity, making up **51% of all internet traffic** in 2024. Bad bots alone account for **37%** — rising for the sixth consecutive year.

**Human Security State of AI Traffic Report (March 2026):** Automated traffic grew **eight times faster** than human activity in 2025. AI traffic specifically increased **187%** from January to December 2025. Traffic from AI agents (like OpenClaw) grew **nearly 8,000%** in 2025.

**Cloudflare CEO Matthew Prince** (SXSW, March 2026): The internet was ~20% bot traffic pre-generative AI, mostly from Google's crawler. He predicted AI bots would exceed human traffic by 2027, citing *"the rise of generative AI and its just insatiable need for data."*

### The Bot Spectrum

Not all bot traffic is malicious:
- **Good bots:** Google's web crawler, AI Overviews, autofill, accessibility tools
- **Bad bots:** Credential stuffing, content scraping, DDoS, click fraud
- **AI agents:** OpenClaw and similar autonomous agents performing actions for users

**Human Security CEO Stu Solomon:** *"The Internet as a whole was created with this very basic notion that there's a human being on the other side of the computer screen, and that notion is very rapidly being replaced."*

### Measurement Challenges

Prof. Filippo Menczer (Indiana University): *"You can try to estimate the amount of bot traffic by looking at the agent strings, but these are very noisy estimates. They depend on what sample you get."*

Human Security's report acknowledges: *"While it used user-agent strings to identify AI operators, the reliability of that self-identification is a growing concern."*

---

## 5. The Interconnected Crisis: A Systems View

These four forces form a **self-reinforcing feedback loop**:

```
LLMs train on web content (extraction)
    → Generate massive volumes of content (pollution)
    → Deprioritize original sources in AI search (defunding)
    → Flood sites with bot traffic (overwhelming)
    → Degrading the very content ecosystem they depend on
```

### The Paradox

LLMs need high-quality, human-created content to train on. But they are simultaneously:
1. **Destroying the economic model** that funds content creation (traffic → ad revenue)
2. **Polluting the signal** with AI-generated noise (future models training on AI slop)
3. **Overwhelming infrastructure** with automated traffic (increasing costs for publishers)

This is sometimes called **"model collapse"** at the ecosystem level — the degradation of training data quality as AI-generated content increasingly dominates the web.

### Who Wins, Who Loses

**Winners:**
- AI companies (extracting value from the commons)
- Platforms with proprietary data (Reddit's $60M/year Google deal)
- Brands with strong direct audiences (email, subscriptions)
- Infrastructure providers (Cloudflare selling bot protection)

**Losers:**
- Independent publishers and journalists
- Open-web content creators (ad-supported models)
- Academic integrity
- Users seeking authentic human perspectives
- The web as a trust-based commons

---

## Sources

### Legal / Copyright
- AI Vortex: "AI Copyright Training Data Lawsuits 2026" (July 2026)
- ZwillGen: "How AI is Shaping Web Scraping Litigation"
- National Law Review: "Legal Issues in Data Scraping for AI Training"
- Thomson Reuters v. Ross Intelligence (D. Del., Feb 2025)
- NYT v. OpenAI (S.D.N.Y., in discovery)
- Getty v. Stability AI (D. Del.)
- Bartz v. Meta (N.D. Cal.)

### Bot Traffic
- Human Security: "State of AI Traffic Report" (March 2026)
- Imperva: "2025 Bad Bot Report"
- CNBC: "AI and bots have officially taken over the internet" (March 26, 2026)
- Cloudflare blog: "Perplexity is using stealth, undeclared crawlers" (August 2025)
- Cloudflare CEO Matthew Prince at SXSW (March 2026)

### Traffic / Publisher Impact
- SEO Kreativ: "AI Overviews Traffic 2026: 58% CTR Drop" (May 2026)
- Omnibound: "Zero-Click Search Statistics (2026): 52+ Data Points"
- SparkToro/Datos: "2024 Zero-Click Search Study"
- Bain & Company: "Goodbye Clicks, Hello AI" (February 2025)
- Pew Research Center: AI Overview CTR study (July 2025, n=68,879)
- Digiday: "Google AI Overviews linked to 25% drop in publisher referral traffic"
- Search Engine Journal: "Impact of AI Overviews" (2026)
- Semrush: "26 AI SEO Statistics for 2026"
- Gartner: search volume forecast (February 2024)
- Kellogg Insight: "As AI Eats Web Traffic, Don't Panic — Evolve"

### AI Slop / Content Quality
- Kingy AI: "The AI Slop of 2025" (December 2025)
- Futurism: "Over 50 Percent of the Internet Is Now AI Slop" (October 2025)
- Graphite: AI content analysis report (2025, n=65,000 articles)
- Euronews: "2025 was the year AI slop went mainstream" (December 2025)
- Forbes: "AI Slop Is Flooding Academic Journals" (April 2026)
- Financial Times: AI-generated reviews at ICLR 2025
- American Philosophical Association Blog: "The Thief of Virtue" (December 2025)
- Meltwater: AI slop mention tracking (2025)
- The Guardian: "From shrimp Jesus to erotic tractors" (December 2025)

---
*Report compiled July 20, 2026*
