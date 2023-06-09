// emas: LinkedList<ResEmaAda>,




// 其实ema list倒也不需要，我们可以有个init的user range
// pub struct EmaList {
//     // intrusive linked list of reserve ema node for EMM
//     emm: LinkedList<ResEmaAda>,
//     // intrusive linked list of regular ema node for User
//     user: LinkedList<RegEmaAda>,
// }

// pub enum EmaType {
//     Emm,
//     User,
// }

// impl EmaList {

//     pub fn new() -> Self {
//         Self {
//             emm: LinkedList::new(ResEmaAda::new()),
//             user: LinkedList::new(RegEmaAda::new()),
//         }
//     }

//     pub fn emm_insert(&mut self, ema: Box<EMA<ResAlloc>, ResAlloc>) {

//     }
// }