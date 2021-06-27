This project aims to create a eink-friendly view of Archive of Our Own for Kobo devices, allowing reading and interacting with works without having to save them to local memory first. It also aims to provide functionality that Ao3 has declined to implement on their end, such as the ability to set default search parameters, and the ability to easily access saved searches.

## Pre-emptive FAQ
**Why isn't this being made as an addon to Plato?**  
The structure of Ao3 works don't map particularly well to documents in Plato, which makes shoehorning them in tough, plus this code requires a lot of additional network functionality. But eventually there'll be a passthrough to Ao3's epub download functionality which will let you save and open stuff in Plato later.

**Why is the binary so much larger than Plato?**  
Network and HTML processing packages, mostly. If Ao3 ever releases an API, we'll be able to ditch the latter (as reqwests, the HTTP client package, has hooks for JSON processing), but until then we're stuck scraping full HTML pages and then extracting the necessary bits. But hey, it's still smaller than trying to save an entire tag off into epub and loading that, right?

**Will you make a version for <other site>?**  
No, but you're welcome to fork this and do it yourself! The changes I've already made in the code should make it a lot easier to port for another site, as various views and handlers exist already.


## Known Issues:
* Notification position drifts (issue underlying in Plato)
* Frontlight physical button takes screenshots rather than toggling frontlight (intentional for testing, will be reverted with the UI is more stable)
* Will crash immediately on load if it can't reach Ao3 (either because of lack of wifi or because the site is down)
* Works with anonymous authors will show as having no authors
* Changing reader display settings doesn't alter current reader instances
* Clicking on a tag in the About Work overlay inside the reader does nothing
* About Work overlay currently missing summary/stats items
* About Work overlay pages past the 1st not accessible due to positioning
* Bookmarks icon does nothing.

## To be Done:
* Other views (Author, Series, Bookmarks, Collections)
* Home page
* Search form
* Access to reading/leaving comments
* Work blocklist
* Worklist navigation history
* Login (partially implemented, but getting the auth cycle right has proven tricky)
* Option to set global default search parameters
* Option to save works for offline reading

## Questions:
* Does this render properly on devices other than the Glo?
* About Work overlay info and work items in indexes are currently kind of ugly - how could they be better?
* At the moment works are loaded entirely, to dodge around having to figure out how to handle chapter navigation, but this results in long load times for large works, and no way of telling what chapter you're in - is this acceptable? How can we handle remote chapters?