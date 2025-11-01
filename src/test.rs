
#[cfg(test)]
mod tests {
    use crate::app::{CanvasConnection};
	use crate::board::{Board, svg_reader::SvgBoardInfo};
	use crate::project::Project;
    use std::rc::Rc;
    use std::cell::RefCell;
	use serde_json;

	impl Board {
		pub fn dummy_svg() -> Self {
			let mut board = Board::default();
			board.svg_board_info = Some(SvgBoardInfo::default());
			board
		}
	}

	#[test]
	fn test_board_connection_references() {
		let mut project = Project::default();

		// board rcs
		let b1_rc = Rc::new(Board::dummy_svg());
		let b2_rc = Rc::new(Board::dummy_svg());
		let kb = vec![b1_rc.clone(), b2_rc.clone()];
		
		// canvas board rcs
		let cb1_rc = project.add_board(&b1_rc).unwrap();
		let cb2_rc = project.add_board(&b2_rc).unwrap();
		let cb1_id = cb1_rc.borrow().id;
		let cb2_id = cb2_rc.borrow().id;

		// connection
		let connection_rc = Rc::new(RefCell::new(CanvasConnection::new(cb1_rc.clone(), "test".to_string())));
		connection_rc.borrow_mut().end(cb2_rc.clone(), "test2".to_string());
		project.add_connection(&connection_rc);

		// test uuids are correctly assigned
		let conn = project.connections.first().unwrap().borrow();
		let sb_id = conn.get_start_board().borrow().id;
		let eb_id = conn.get_end_board().unwrap().borrow().id;

		// ensure the UUIDs stored within the project reflect objects
		// if so, deserialization will pick them back up and generate runtime references
		assert_eq!(sb_id, cb1_id);
		assert_eq!(eb_id, cb2_id);

		// ser/de
		let serialized = serde_json::to_string(&project).expect("Could not serialize.");
		let mut deserialized: Project = serde_json::from_str(&serialized).expect("Could not deserialize.");
		deserialized.load_board_resources(&kb);

		// the rcs are purely runtime, they came from uuid
		let conn = deserialized.connections.first().unwrap().borrow();
		let sb_rc = conn.get_start_board();
		let eb_rc = conn.get_end_board().unwrap();
		let sb_id = sb_rc.borrow().id;
		let eb_id = eb_rc.borrow().id;

		// first we check that the references are different, aka different objects
		assert!(!Rc::ptr_eq(&sb_rc, &cb1_rc));
		assert!(!Rc::ptr_eq(&eb_rc, &cb2_rc));
		// then we can verify they are the same (same uuid)
		assert_eq!(sb_id, cb1_id);
		assert_eq!(eb_id, cb2_id);
	}
}