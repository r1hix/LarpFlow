"""Unit tests for draft validation, rules, and length calculations."""
from larpflow.core import drafts


def test_tweet_len_plain_text():
    assert drafts.tweet_len("Hello world") == 11


def test_tweet_len_url_expansion():
    # Links on X are counted as 23 characters regardless of raw length
    raw = "Check out https://github.com/r1hix/larpflow for details"
    expected = len("Check out ") + 23 + len(" for details")
    assert drafts.tweet_len(raw) == expected


def test_tweet_len_placeholder_expansion():
    # {REPO_URL} placeholder is counted as 23 characters
    raw = "Repository: {REPO_URL}"
    expected = len("Repository: ") + 23
    assert drafts.tweet_len(raw) == expected


def test_tweet_problems_count():
    assert len(drafts.tweet_problems(["t1", "t2", "t3"])) > 0
    assert "need exactly 4 tweets" in drafts.tweet_problems(["t1", "t2", "t3"])[0]


def test_tweet_problems_empty_or_too_long():
    good_tweet = "A" * 100
    long_tweet = "A" * 280
    issues = drafts.tweet_problems([good_tweet, "", good_tweet, long_tweet])
    assert any("tweet 2 is empty" in i for i in issues)
    assert any("tweet 4 is 280 characters" in i for i in issues)


def test_normalize_platforms():
    assert drafts.normalize_platforms(None) is None
    assert drafts.normalize_platforms("") is None
    assert drafts.normalize_platforms("x,linkedin") == {"x", "linkedin"}
    assert drafts.normalize_platforms("twitter,discord") == {"x", "discord"}
    assert drafts.normalize_platforms(["tweet", "reddit"]) == {"x", "reddit"}
    assert drafts.normalize_platforms("invalid_platform") is None


def test_platform_problems_linkedin():
    # Short post (< 140 words)
    d = drafts.Drafts(
        tweets=["1", "2", "3", "4"],
        linkedin="This is way too short of a linkedin post.",
        reddit_title="Title",
        reddit_body="### The Bottleneck\n### What Failed\n### The Fix\nContent",
        reddit_subreddit="SideProject",
    )
    issues = drafts.platform_problems(d, platforms="linkedin")
    assert "linkedin" in issues
    assert any("need 140 to 350" in i for i in issues["linkedin"])


def test_platform_problems_reddit_missing_headers():
    d = drafts.Drafts(
        reddit_title="Test title",
        reddit_body="Just some plain body without required headers.",
        reddit_subreddit="SideProject",
    )
    issues = drafts.platform_problems(d, platforms="reddit")
    assert "reddit" in issues
    assert any("The Bottleneck" in i for i in issues["reddit"])
    assert any("The Fix" in i for i in issues["reddit"])


def test_platform_problems_reddit_no_links_rule():
    d = drafts.Drafts(
        reddit_title="Test title",
        reddit_body="### The Bottleneck\n### What Failed\n### The Fix\nVisit https://example.com for more",
        reddit_subreddit="SideProject",
    )
    issues = drafts.platform_problems(d, platforms="reddit")
    assert "reddit" in issues
    assert any("must not contain links" in i for i in issues["reddit"])
