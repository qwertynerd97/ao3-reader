use super::Overlay;
use super::{View, Event, Hub, Bus, Id, ID_FEEDER, RenderQueue, ViewId, RenderData};
use crate::view::MINI_BAR_HEIGHT;
use crate::view::works::work::{Work, WorkView};
use crate::view::tag::Tag;
use crate::font::{Fonts, LABEL_STYLE};
use crate::framebuffer::Framebuffer;
use super::{BORDER_RADIUS_MEDIUM, CLOSE_IGNITION_DELAY, SMALL_BAR_HEIGHT, BIG_BAR_HEIGHT};
use crate::unit::scale_by_dpi;
use crate::app::Context;
use crate::device::CURRENT_DEVICE;
use crate::geom::{Rectangle, CycleDir};
use crate::document::Location;

#[derive(Clone)]
pub struct WorksOverlay {
    id: Id,
    children: Vec<Box<dyn View>>,
    view_id: ViewId,
    overlay: Overlay
}

impl WorksOverlay {
    pub fn new(context: &mut Context) -> WorksOverlay{
        let id = ID_FEEDER.next();
        let mut overlay = Overlay::new(ViewId::Overlay, context);
        let dpi = CURRENT_DEVICE.dpi;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let big_height = scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32;
        let mini_height = scale_by_dpi(MINI_BAR_HEIGHT, dpi) as i32;
        let padding = scale_by_dpi(BORDER_RADIUS_MEDIUM, dpi) as i32;
        let mut children = Vec::new();
        children.push(Box::new(overlay.clone()) as Box<dyn View>);

        let (width, height) = context.display.dims;
        let dy = small_height + padding;

        let msg_rect = rect![0,
        dy + small_height,
        width as i32,
        dy + big_height + small_height ];

let msg_rect2 = rect![0,
        dy + big_height + small_height,
        width as i32,
        dy + 3 * big_height + small_height];


        let d = r###"
        <li id="work_25413577" class="work blurb group work-25413577 user-1959271" role="article">
                        <!--title, author, fandom-->
                        <div class="header module">
                            <h4 class="heading">
                                <a href="/works/25413577">Dear Stranger, I love you</a>
                                by
                                <!-- do not cache -->
                                <a rel="author" href="/users/Emberfire31/pseuds/Emberfire31">Emberfire31</a>
                            </h4>
                            <h5 class="fandoms heading">
                                <span class="landmark">Fandoms:</span>
                                <a class="tag" href="/tags/%E9%95%87%E9%AD%82%20%7C%20Guardian%20(TV)/works">镇魂 | Guardian (TV)</a>
                                &nbsp;
                            </h5>
                            <!--required tags-->
                            <ul class="required-tags">
                                <li> <a class="help symbol question modal" title="Symbols key" aria-controls="#modal" href="/help/symbols-key.html"><span class="rating-mature rating" title="Mature"><span class="text">Mature</span></span></a></li>
                                <li> <a class="help symbol question modal" title="Symbols key" aria-controls="#modal" href="/help/symbols-key.html"><span class="warning-choosenotto warnings" title="Choose Not To Use Archive Warnings"><span class="text">Choose Not To Use Archive Warnings</span></span></a></li>
                                <li> <a class="help symbol question modal" title="Symbols key" aria-controls="#modal" href="/help/symbols-key.html"><span class="category-slash category" title="M/M"><span class="text">M/M</span></span></a></li>
                                <li> <a class="help symbol question modal" title="Symbols key" aria-controls="#modal" href="/help/symbols-key.html"><span class="complete-no iswip" title="Work in Progress"><span class="text">Work in Progress</span></span></a></li>
                            </ul>
                            <p class="datetime">16 May 2021</p>
                        </div>
                        <!--warnings again, cast, freeform tags-->
                        <h6 class="landmark heading">Tags</h6>
                        <ul class="tags commas">
                            <li class='warnings'><strong><a class="tag" href="/tags/Choose%20Not%20To%20Use%20Archive%20Warnings/works">Creator Chose Not To Use Archive Warnings</a></strong></li>
                            <li class='relationships'><a class="tag" href="/tags/WeiLan%20-%20Relationship/works">WeiLan - Relationship</a></li>
                            <li class='relationships'><a class="tag" href="/tags/Shen%20Wei*s*Zhao%20Yunlan/works">Shen Wei/Zhao Yunlan</a></li>
                            <li class='characters'><a class="tag" href="/tags/Da%20Ching/works">Da Ching</a></li>
                            <li class='characters'><a class="tag" href="/tags/Sh%C4%9Bn%20W%C4%93i/works">Shěn Wēi</a></li>
                            <li class='characters'><a class="tag" href="/tags/Zhao%20Yunlan/works">Zhao Yunlan</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Pen%20Pals/works">Pen Pals</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Double%20Life/works">Double Life</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Love%20Letters/works">Love Letters</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/love%20square/works">love square</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Idiots%20in%20Love/works">Idiots in Love</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Friends%20to%20Lovers/works">Friends to Lovers</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Strangers%20to%20Lovers/works">Strangers to Lovers</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Healing/works">Healing</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Pining/works">Pining</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/hidden%20identity/works">hidden identity</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Other%20Additional%20Tags%20to%20Be%20Added/works">Other Additional Tags to Be Added</a></li>
                        </ul>
                        <!--summary-->
                        <h6 class="landmark heading">Summary</h6>
                        <blockquote class="userstuff summary">
                            <p>When the University where Shen Wei works organizes a secret pen pal experiment, his colleagues immediately write down his name as a participant.</p>
                            <p>He doubts that he will go further than the four mandatory letters and he is mostly going to assuage the professors.</p>
                            <p>However, his secret draw seems to be a very bright and interesting man who just begs to be loved.</p>
                            <p>They both begin to share secrets and words that have never been uttered out loud and their relationship grows stronger by the letter.</p>
                            <p>However, one day Shen Wei meets Zhao Yunlan again, and whilst he is ready to drop his pen pal relationship, his pen pal has also met a very cute and weird professor and he's falling in love quickly!</p>
                            <p> </p>
                            <p>Yes. This is a love square!</p>
                        </blockquote>
                        <!--stats-->
                        <dl class="stats">
                            <dt class="language">Language:</dt>
                            <dd class="language">English</dd>
                            <dt class="words">Words:</dt>
                            <dd class="words">28,178</dd>
                            <dt class="chapters">Chapters:</dt>
                            <dd class="chapters"><a href="/works/25413577/chapters/77473271">8</a>/?</dd>
                            <dt class="comments">Comments:</dt>
                            <dd class="comments"><a href="/works/25413577?show_comments=true&amp;view_full_work=true#comments">53</a></dd>
                            <dt class="kudos">Kudos:</dt>
                            <dd class="kudos"><a href="/works/25413577?view_full_work=true#kudos">83</a></dd>
                            <dt class="bookmarks">Bookmarks:</dt>
                            <dd class="bookmarks"><a href="/works/25413577/bookmarks">13</a></dd>
                            <dt class="hits">Hits:</dt>
                            <dd class="hits">1232</dd>
                        </dl>
                    </li>
        "###;

        let d2 = r###"
        <li id="work_15939905" class="work blurb group work-15939905 user-2330350" role="article">
                        <!--title, author, fandom-->
                        <div class="header module">
                            <h4 class="heading">
                                <a href="/works/15939905">Crazy Rich Boyfriend</a>
                                by
                                <!-- do not cache -->
                                <a rel="author" href="/users/thekunlundilemmas/pseuds/thekunlundilemmas">thekunlundilemmas</a>
                            </h4>
                            <h5 class="fandoms heading">
                                <span class="landmark">Fandoms:</span>
                                <a class="tag" href="/tags/%E9%95%87%E9%AD%82%20%7C%20Guardian%20(TV)/works">镇魂 | Guardian (TV)</a>
                                &nbsp;
                            </h5>
                            <!--required tags-->
                            <ul class="required-tags">
                                <li> <a class="help symbol question modal" title="Symbols key" aria-controls="#modal" href="/help/symbols-key.html"><span class="rating-explicit rating" title="Explicit"><span class="text">Explicit</span></span></a></li>
                                <li> <a class="help symbol question modal" title="Symbols key" aria-controls="#modal" href="/help/symbols-key.html"><span class="warning-choosenotto warnings" title="Choose Not To Use Archive Warnings"><span class="text">Choose Not To Use Archive Warnings</span></span></a></li>
                                <li> <a class="help symbol question modal" title="Symbols key" aria-controls="#modal" href="/help/symbols-key.html"><span class="category-slash category" title="M/M"><span class="text">M/M</span></span></a></li>
                                <li> <a class="help symbol question modal" title="Symbols key" aria-controls="#modal" href="/help/symbols-key.html"><span class="complete-no iswip" title="Work in Progress"><span class="text">Work in Progress</span></span></a></li>
                            </ul>
                            <p class="datetime">16 May 2021</p>
                        </div>
                        <!--warnings again, cast, freeform tags-->
                        <h6 class="landmark heading">Tags</h6>
                        <ul class="tags commas">
                            <li class='warnings'><strong><a class="tag" href="/tags/Choose%20Not%20To%20Use%20Archive%20Warnings/works">Creator Chose Not To Use Archive Warnings</a></strong></li>
                            <li class='relationships'><a class="tag" href="/tags/Shen%20Wei*s*Zhao%20Yunlan/works">Shen Wei/Zhao Yunlan</a></li>
                            <li class='relationships'><a class="tag" href="/tags/Da%20Qing*s*Lin%20Jing%20(Guardian)/works">Da Qing/Lin Jing (Guardian)</a></li>
                            <li class='characters'><a class="tag" href="/tags/Sh%C4%9Bn%20W%C4%93i/works">Shěn Wēi</a></li>
                            <li class='characters'><a class="tag" href="/tags/Zhao%20Yunlan/works">Zhao Yunlan</a></li>
                            <li class='characters'><a class="tag" href="/tags/L%C3%ADn%20J%C3%ACng/works">Lín Jìng</a></li>
                            <li class='characters'><a class="tag" href="/tags/Guo%20Changcheng/works">Guo Changcheng</a></li>
                            <li class='characters'><a class="tag" href="/tags/Y%C3%A8%20Z%C5%ABn/works">Yè Zūn</a></li>
                            <li class='characters'><a class="tag" href="/tags/Zh%C3%B9%20H%C3%B3ng/works">Zhù Hóng</a></li>
                            <li class='characters'><a class="tag" href="/tags/Chu%20Shu%20Zhi/works">Chu Shu Zhi</a></li>
                            <li class='characters'><a class="tag" href="/tags/W%C4%81ng%20Zh%C4%93ng/works">Wāng Zhēng</a></li>
                            <li class='characters'><a class="tag" href="/tags/Da%20Qing%20(Guardian)/works">Da Qing (Guardian)</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Alternate%20Universe%20-%20Modern%20Setting/works">Alternate Universe - Modern Setting</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/inspired%20by%20the%20crazy%20rich%20asians/works">inspired by the crazy rich asians</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Alternate%20Universe%20-%20No%20Powers/works">Alternate Universe - No Powers</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/professor!shenwei/works">professor!shenwei</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/chief!yunlan/works">chief!yunlan</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/model!daxing/works">model!daxing</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/scientist!linjing/works">scientist!linjing</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/rich!shenwei/works">rich!shenwei</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/cousin!chu/works">cousin!chu</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/officer!guo/works">officer!guo</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/but%20at%20the%20same%20time%20it%20aint%20really%20like%20cra/works">but at the same time it aint really like cra</a></li>
                        </ul>
                        <!--summary-->
                        <h6 class="landmark heading">Summary</h6>
                        <blockquote class="userstuff summary">
                            <p>In which Zhao Yunlan finds out that his boyfriend is a billionaire and that he was going to meet his crazy rich family.</p>
                            <p>Or in other words:</p>
                            <p>It's the crazy rich asians but gayer.</p>
                        </blockquote>
                        <!--stats-->
                        <dl class="stats">
                            <dt class="language">Language:</dt>
                            <dd class="language">English</dd>
                            <dt class="words">Words:</dt>
                            <dd class="words">1,830</dd>
                            <dt class="chapters">Chapters:</dt>
                            <dd class="chapters"><a href="/works/15939905/chapters/77486630">2</a>/?</dd>
                            <dt class="comments">Comments:</dt>
                            <dd class="comments"><a href="/works/15939905?show_comments=true&amp;view_full_work=true#comments">10</a></dd>
                            <dt class="kudos">Kudos:</dt>
                            <dd class="kudos"><a href="/works/15939905?view_full_work=true#kudos">101</a></dd>
                            <dt class="bookmarks">Bookmarks:</dt>
                            <dd class="bookmarks"><a href="/works/15939905/bookmarks">15</a></dd>
                            <dt class="hits">Hits:</dt>
                            <dd class="hits">1604</dd>
                        </dl>
                    </li>"###;

                    let d3 = r###"
                    <li id="work_31167851" class="work blurb group work-31167851 user-3321627" role="article">
                        <!--title, author, fandom-->
                        <div class="header module">
                            <h4 class="heading">
                                <a href="/works/31167851">Skies So Riddled, We Can't See The Stars</a>
                                by
                                <!-- do not cache -->
                                <a rel="author" href="/users/Redsilkthread/pseuds/TaroHopia">TaroHopia (Redsilkthread)</a>
                            </h4>
                            <h5 class="fandoms heading">
                                <span class="landmark">Fandoms:</span>
                                <a class="tag" href="/tags/%E9%95%87%E9%AD%82%20%7C%20Guardian%20-%20priest/works">镇魂 | Guardian - priest</a>, <a class="tag" href="/tags/%E9%95%87%E9%AD%82%20%7C%20Guardian%20(TV%202018)/works">镇魂 | Guardian (TV 2018)</a>
                                &nbsp;
                            </h5>
                            <!--required tags-->
                            <ul class="required-tags">
                                <li> <a class="help symbol question modal" title="Symbols key" aria-controls="#modal" href="/help/symbols-key.html"><span class="rating-teen rating" title="Teen And Up Audiences"><span class="text">Teen And Up Audiences</span></span></a></li>
                                <li> <a class="help symbol question modal" title="Symbols key" aria-controls="#modal" href="/help/symbols-key.html"><span class="warning-yes warnings" title="Graphic Depictions Of Violence"><span class="text">Graphic Depictions Of Violence</span></span></a></li>
                                <li> <a class="help symbol question modal" title="Symbols key" aria-controls="#modal" href="/help/symbols-key.html"><span class="category-slash category" title="M/M"><span class="text">M/M</span></span></a></li>
                                <li> <a class="help symbol question modal" title="Symbols key" aria-controls="#modal" href="/help/symbols-key.html"><span class="complete-no iswip" title="Work in Progress"><span class="text">Work in Progress</span></span></a></li>
                            </ul>
                            <p class="datetime">16 May 2021</p>
                        </div>
                        <!--warnings again, cast, freeform tags-->
                        <h6 class="landmark heading">Tags</h6>
                        <ul class="tags commas">
                            <li class='warnings'><strong><a class="tag" href="/tags/Graphic%20Depictions%20Of%20Violence/works">Graphic Depictions Of Violence</a></strong></li>
                            <li class='relationships'><a class="tag" href="/tags/Shen%20Wei*s*Zhao%20Yunlan/works">Shen Wei/Zhao Yunlan</a></li>
                            <li class='relationships'><a class="tag" href="/tags/Da%20Qing%20*a*%20Zhao%20Yunlan/works">Da Qing &amp; Zhao Yunlan</a></li>
                            <li class='relationships'><a class="tag" href="/tags/Da%20Qing%20*a*%20Ye%20Zun%20(Guardian)/works">Da Qing &amp; Ye Zun (Guardian)</a></li>
                            <li class='relationships'><a class="tag" href="/tags/Ye%20Zun%20*a*%20Zhao%20Yunlan/works">Ye Zun &amp; Zhao Yunlan</a></li>
                            <li class='relationships'><a class="tag" href="/tags/Shen%20Wei%20*a*%20Ye%20Zun%20(Guardian)/works">Shen Wei &amp; Ye Zun (Guardian)</a></li>
                            <li class='characters'><a class="tag" href="/tags/Zhao%20Yunlan/works">Zhao Yunlan</a></li>
                            <li class='characters'><a class="tag" href="/tags/Shen%20Wei%20(Guardian)/works">Shen Wei (Guardian)</a></li>
                            <li class='characters'><a class="tag" href="/tags/Ye%20Zun%20(Guardian)/works">Ye Zun (Guardian)</a></li>
                            <li class='characters'><a class="tag" href="/tags/Da%20Qing%20(Guardian)/works">Da Qing (Guardian)</a></li>
                            <li class='characters'><a class="tag" href="/tags/Special%20Investigation%20Division%20%7C%20SID%20Ensemble%20(Guardian)/works">Special Investigation Division | SID Ensemble (Guardian)</a></li>
                            <li class='characters'><a class="tag" href="/tags/Shen%20Xi%20%7C%20Zhao%20Yunlan&#39;s%20Mother/works">Shen Xi | Zhao Yunlan&#39;s Mother</a></li>
                            <li class='characters'><a class="tag" href="/tags/cameos%20from%20major*s*minor%20FF7R%20characters/works">cameos from major/minor FF7R characters</a></li>
                            <li class='characters'><a class="tag" href="/tags/Original%20Chocobo%20Character(s)%20(Final%20Fantasy)/works">Original Chocobo Character(s) (Final Fantasy)</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Additional%20Warnings%20In%20Author&#39;s%20Note/works">Additional Warnings In Author&#39;s Note</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Established%20Relationship/works">Established Relationship</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Angst/works">Angst</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Canon-Typical%20Violence/works">Canon-Typical Violence</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Alternate%20Universe%20-%20Fantasy/works">Alternate Universe - Fantasy</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Found%20Family%20Dynamics/works">Found Family Dynamics</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Zh%C3%A0o%20Y%C3%BAnl%C3%A1n%20is%20a%20Cetra/works">Zhào Yúnlán is a Cetra</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Ye%20Zun%20works%20for%20Avalanche/works">Ye Zun works for Avalanche</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Shen%20Wei%20works%20for%20SHINRA/works">Shen Wei works for SHINRA</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/and%20he%20isn&#39;t%20happy%20about%20it/works">and he isn&#39;t happy about it</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Depression/works">Depression</a></li>
                            <li class='freeforms'><a class="tag" href="/tags/Guilt/works">Guilt</a></li>
                        </ul>
                        <!--summary-->
                        <h6 class="landmark heading">Summary</h6>
                        <blockquote class="userstuff summary">
                            <p>With their lives uprooted and Shen Wei trapped in Midgar, Zhao Yunlan tries to make sense of his new world and discovers there is a much larger threat than the one that separated him from Shen Wei. The world is vast, as are its mysteries and Zhao Yunlan and Ye Zun will have to uncover long-hidden secrets that span countless lifetimes and beyond.</p>
                            <p>A Guardian/Final Fantasy 7 fusion.</p>
                        </blockquote>
                        <h6 class="landmark heading">Series</h6>
                        <ul class="series">
                            <li>
                                Part <strong>6</strong> of <a href="/series/1908178">From the Slums to the Never Ending Sky</a>
                            </li>
                        </ul>
                        <!--stats-->
                        <dl class="stats">
                            <dt class="language">Language:</dt>
                            <dd class="language">English</dd>
                            <dt class="words">Words:</dt>
                            <dd class="words">7,947</dd>
                            <dt class="chapters">Chapters:</dt>
                            <dd class="chapters"><a href="/works/31167851/chapters/77448797">2</a>/?</dd>
                            <dt class="comments">Comments:</dt>
                            <dd class="comments"><a href="/works/31167851?show_comments=true&amp;view_full_work=true#comments">11</a></dd>
                            <dt class="kudos">Kudos:</dt>
                            <dd class="kudos"><a href="/works/31167851?view_full_work=true#kudos">8</a></dd>
                            <dt class="bookmarks">Bookmarks:</dt>
                            <dd class="bookmarks"><a href="/works/31167851/bookmarks">3</a></dd>
                            <dt class="hits">Hits:</dt>
                            <dd class="hits">55</dd>
                        </dl>
                    </li>
                    "###;

            let work = Work::new(msg_rect, d2.to_string(), 1, false, WorkView::Short);
            let work2 = Work::new(msg_rect2, d3.to_string(), 2, false, WorkView::Long);
            let msg_rect3 = rect![400,
            dy + 3 * big_height + small_height,
            width as i32,
            dy + 3 * big_height + mini_height + small_height];
        //     let tag = Tag::new(msg_rect3, "With their lives uprooted and Shen Wei trapped in Midgar, Zhao Yunlan tries to make sense of his new world and discovers there is a m".to_string(),
        // true, width as i32, 0, Location::Uri("https://archiveofourown.org/works/25413577".to_string()), &mut context.fonts);

            //children.push(Box::new(work.clone()) as Box<dyn View>);
            //children.push(Box::new(work2) as Box<dyn View>);
            //children.push(Box::new(tag) as Box<dyn View>);

            // let mut start_x = 0;
            // let mut start_y = dy; // + 3 * big_height + mini_height + small_height;

            // for tag in work.info.tags {
            //     println!("Start x is {}, start y is {}", start_x, start_y);
            //     let tag_rect = rect![start_x, start_y, width as i32, start_y + mini_height];
            //     let tag = Tag::new(tag_rect, tag.title, width as i32, 0, Some(Location::Uri(tag.location)), &mut context.fonts, LABEL_STYLE);
            //     let end_pt = tag.end_point();
            //     let lines = tag.lines();
            //     let rem_width = width as i32 - end_pt.x;
            //     println!("lines is {}, remaining width is {}", lines, rem_width);
            //     if rem_width < mini_height {
            //         start_y += lines as i32 * mini_height;
            //         start_x = 0;
            //     } else {
            //         start_x = end_pt.x + padding;
            //         start_y += (lines as i32 - 1 ) * mini_height;
            //     }

            //     children.push(Box::new(tag) as Box<dyn View>);
            // }

            WorksOverlay {
                id,
                children,
                view_id: ViewId::Overlay,
                overlay
            }
    }
}

impl View for WorksOverlay {
    fn handle_event(&mut self, evt: &Event, hub: &Hub, bus: &mut Bus, rq: &mut RenderQueue, context: &mut Context) -> bool {
        match *evt {
            Event::Gesture(..) => true,
            _ => self.overlay.handle_event(evt, hub, bus, rq, context),
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, fonts: &mut Fonts) {
        self.overlay.render(fb, rect, fonts);
    }

    fn rect(&self) -> &Rectangle {
        &self.overlay.rect()
    }

    fn rect_mut(&mut self) -> &mut Rectangle {
        self.overlay.rect_mut()
    }

    fn children(&self) -> &Vec<Box<dyn View>> {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn View>> {
        &mut self.children
    }

    fn id(&self) -> Id {
        self.id
    }

    fn view_id(&self) -> Option<ViewId> {
        Some(self.view_id)
    }
}