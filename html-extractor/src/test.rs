use html_extractor::HtmlExtractor;

#[test]
fn test() {
    let data = TestData::extract_from_str(
        r#"
            <div id="data1">
                <div class="data1-1">1</div>
            </div>
            <div id="data2">2</div>
            <div id="data3" data-3="3"></div>
            <div id="data4">
                <div>
                    <div class="data1-1">1</div>
                </div>
                <div>
                    <div class="data1-1">2</div>
                </div>
                <div>
                    <div class="data1-1">3</div>
                </div>
                <div>
                    <div class="data1-1">4</div>
                </div>
            </div>
            <div id="data5">
                <div>1</div>
                <div>2</div>
                <div>3</div>
                <div>4</div>
            </div>
            <div id="data6">
                <div data-6="1"></div>
                <div data-6="2"></div>
                <div data-6="3"></div>
                <div data-6="4"></div>
            </div>

            <div id="data7">%%%7%%%</div>
            <div id="data8" data-8="%%%8%%%"></div>
            <div id="data9">
                <div>ignore<br />%%%1%%%5%%%</div>
                <div>ignore<br />%%%2%%%6%%%</div>
                <div>ignore<br />%%%3%%%7%%%</div>
                <div>ignore<br />%%%4%%%8%%%</div>
            </div>
            <div id="data10">
                <div data-10="%%%1%%%5%%%"></div>
                <div data-10="%%%2%%%6%%%"></div>
                <div data-10="%%%3%%%7%%%"></div>
                <div data-10="%%%4%%%8%%%"></div>
            </div>

            <div id="data11">ignore<br />ignore<br />%%%7%%%27%%%</div>
            <div id="data12" data-12="%%%8%%%18%%%46%%%"></div>
            <div id="data13">
                <div>%%%1%%%5%%%9%%%13%%%</div>
                <div>%%%2%%%6%%%10%%%14%%%</div>
                <div>%%%3%%%7%%%11%%%15%%%</div>
                <div>%%%4%%%8%%%12%%%16%%%</div>
            </div>
            <div id="data14">
                <div data-14="%%%1%%%5%%%9%%%13%%%17%%%"></div>
                <div data-14="%%%2%%%6%%%10%%%14%%%18%%%"></div>
                <div data-14="%%%3%%%7%%%11%%%15%%%19%%%"></div>
                <div data-14="%%%4%%%8%%%12%%%16%%%20%%%"></div>
            </div>
        "#,
    )
    .unwrap();

    assert_eq!(data, TestData {
        data1: InnerData { data1_1: 1 },
        data2: 2,
        data3: 3,
        data4: vec![InnerData { data1_1: 1 }, InnerData { data1_1: 2 }, InnerData { data1_1: 3 }, InnerData { data1_1: 4 }],
        data5: vec![1, 2, 3, 4],
        data6: vec![1, 2, 3, 4],
        data7: 7,
        data8: 8,
        data9: vec![(1, 5), (2, 6), (3, 7), (4, 8)],
        data10: vec![(1, 5), (2, 6), (3, 7), (4, 8)],
        data11_1: 7,
        data11_2: 27,
        data12_1: 8,
        data12_2: 18,
        data12_3: 46,
        data13: vec![(1, 5, 9, 13), (2, 6, 10, 14), (3, 7, 11, 15), (4, 8, 12, 16)],
        data14: vec![(1, 5, 9, 13, 17), (2, 6, 10, 14, 18), (3, 7, 11, 15, 19), (4, 8, 12, 16, 20)],
    });
}
html_extractor::html_extractor! {
    #[derive(Debug, PartialEq)]
    pub TestData {
        pub(crate) data1: InnerData = (elem of "#data1"),
        pub(super) data2: usize = (text of "#data2"),
        pub data3: usize = (attr["data-3"] of "#data3"),

        data4: Vec<InnerData> = (elem of "#data4 > div", collect),
        data5: Vec<usize> = (text of "#data5 > div", collect),
        data6: Vec<usize> = (attr["data-6"] of "#data6 > div", collect),

        (data7: usize,) = (text of "#data7", capture with "%%%(.*)%%%"),
        (data8: usize,) = (attr["data-8"] of "#data8", capture with "%%%(.*)%%%"),

        data9: Vec<(usize, usize)> = (text[1] of "#data9 > div", capture with "%%%(.*)%%%(.*)%%%", collect),
        data10: Vec<(usize, usize)> = (attr ["data-10"] of "#data10 > div", capture with "%%%(.*)%%%(.*)%%%", collect),

        (data11_1: usize, data11_2: usize) = (text[2] of "#data11", capture with "%%%(.*)%%%(.*)%%%"),
        (data12_1: usize, data12_2: usize, data12_3: usize) = (attr["data-12"] of "#data12", capture with "%%%(.*)%%%(.*)%%%(.*)%%%"),

        data13: Vec<(usize, usize, usize, usize)> = (text of "#data13 > div", capture with "%%%(.*)%%%(.*)%%%(.*)%%%(.*)%%%", collect),
        data14: Vec<(usize, usize, usize, usize, usize)> = (attr["data-14"] of "#data14 > div", capture with "%%%(.*)%%%(.*)%%%(.*)%%%(.*)%%%(.*)%%%", collect),
    }
    #[derive(Debug, PartialEq)]
    pub(crate) InnerData {
        data1_1: usize = (text of ".data1-1")
    }
}