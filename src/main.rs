#![allow(dead_code)] //Атрибут, который убирает проверку на неиспользуемый код

use fltk::{
    app,
    button::Button, //Кнопка
    enums::Event, //Перечисления
    frame::Frame, //Рамка
    prelude::*,
    tree::{Tree, TreeItem, TreeReason, TreeSelect}, //Дерево
    window::Window,
};
use std::cell::RefCell; //Внутренняя изменяемость
use std::env;
use std::rc::Rc; //Разрешаем множественное владение 

#[derive(PartialEq)]
enum State { //enum - перечисления
    MovingUp, //вверх
    MovingDown, //вниз
    Undefined, //не определенно 
}

fn verify_open_till_root(opt: &Option<fltk::tree::TreeItem>) -> bool {  //Проверяет открытие, возвращая бул переменную (true/false)
    let mut par = opt.clone();
    loop { 
        match par.as_ref().unwrap().parent() { 
            Some(p) => { 
                if p.is_close() {
                    return false;
                } else {
                    par = Some(p.clone());
                }
            }
            None => return true, 
        }
    }
}

struct TreeMouseFocus { //Фокус мышки на дереве
    t_widget: fltk::tree::Tree,
    previous_focus: Rc<RefCell<Option<TreeItem>>>,
}

impl TreeMouseFocus { //Блок реализации TreeMouseFocus
    fn new(x: i32, y: i32, width: i32, height: i32, title: &'static str) -> Self { //Задаем параметры в (..) -> возвращаем ссылку
        let mut t_widget = Tree::new(x, y, width, height, title); //Изменяемая переменная t_widget; присваиваем пустой переменной - Tree::new, в которую помещаем (..)
        let previous_focus = Rc::new(RefCell::new(None::<TreeItem>));
        let pfr = Rc::clone(&previous_focus);
        t_widget.set_callback_reason(TreeReason::Selected);
        t_widget.set_callback(|_t| println!("Вы щелкнули по элементу."));
        t_widget.handle(move |t, e| match e {
            Event::Move => {
                let (_, mouse_y) = app::event_coords();
                let mut state = State::Undefined;
                let mut pf = pfr.borrow_mut();
                loop {
                    match &*pf {
                        Some(item) => { //Some - элемент найден
                            let item_y = item.y();
                            match state { //Прописываем фокус мышки
                                State::MovingUp => {
                                    if verify_open_till_root(&pf) {
                                        if mouse_y < item_y {
                                            *pf = pf.as_ref().unwrap().prev();
                                            continue;
                                        };
                                        break;
                                    } else {
                                        *pf = pf.as_ref().unwrap().prev();
                                        continue;
                                    }
                                }
                                State::MovingDown => {
                                    if verify_open_till_root(&pf) {
                                        if mouse_y > item_y + item.h() {
                                            *pf = pf.as_ref().unwrap().next();
                                            continue;
                                        };
                                        break;
                                    } else {
                                        *pf = pf.as_ref().unwrap().next();
                                        continue;
                                    }
                                }
                                State::Undefined => {
                                    if mouse_y < item_y {
                                        *pf = pf.as_ref().unwrap().prev();
                                        state = State::MovingUp;
                                        continue;
                                    };
                                    if mouse_y > item_y + item.h() {
                                        *pf = pf.as_ref().unwrap().next();
                                        state = State::MovingDown;
                                        continue;
                                    };
                                    return true; // Если в том же диапазоне, не обновляйте previous_focus
                                }
                            }
                        }
                        // Окажемся здесь, есть y находится вне дерева, или элемент дерева не присутсвует 
                        None => match &state { //None - элемент не найден 
                            State::MovingUp | State::MovingDown => return true,
                            State::Undefined => {*pf = t.first();
                                state = State::MovingDown;
                                if pf.is_none() {
                                    return true;
                                }
                                continue;
                            }
                        },
                    };
                }
                if verify_open_till_root(&pf) { //Фокус на элементе 
                    t.take_focus().ok();
                    t.set_item_focus(pf.as_ref().unwrap());
                    println!("Установлен фокус на элементе: {:?}", pf.as_ref().unwrap().label());
                }
                true
            }
            _ => false,
        });
        Self { //ссылка
            t_widget,
            previous_focus,
        }
    }

    fn add(&mut self, path: &str) -> Option<TreeItem> {
        self.t_widget.add(path)
    }

    /*Переменная previous focus должна быть установлена None, тк в 
    противном случае может попробовать обратиться к уже освобожденной
    ячейке памяти, когда элемент дерева будет удален*/
    fn remove(&mut self, item: &TreeItem) -> Result<(), FltkError> {
        *self.previous_focus.borrow_mut() = None;
        self.t_widget.remove(item)
    }

    fn get_items(&self) -> Option<Vec<TreeItem>> {
        self.t_widget.get_items()
    }
}

fn main() {
    let path = env::current_dir().unwrap();
    let path: String = path
        .to_str()
        .unwrap()
        .chars()
        .enumerate()
        .map(|(_, c)| match c {
            '\\' => '/', //Изменение пути к window на posix 
            _ => c,
        })
        .collect();
//Задаем парметры для дерева
    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut wind = Window::default().with_size(400, 300);
    let mut but = Button::new(260, 255, 110, 40, "Выбрать строку"); //Кнопка
    let _frame = Frame::new(20, 255, 160, 40, "Первое дерево");
    let mut tree = TreeMouseFocus::new(5, 10, 190, 240, "");
    tree.add(&path);

    let mut items = tree.get_items().unwrap();
    items.as_mut_slice()[0].set_label("/");
//Второе дерево
    let mut tree2 = Tree::new(205, 10, 190, 240, "");
    tree2.set_select_mode(TreeSelect::Multi);
    tree2.add("Первый");
    tree2.add("Первый/1 ступень");
    tree2.add("Первый/2 ступень");
    tree2.add("Второй");
    tree2.add("Третий/1 ступень/1.2 ступень");

    tree2.set_trigger(fltk::enums::CallbackTrigger::ReleaseAlways);

    wind.make_resizable(true);
    wind.show();

    but.set_callback({
        let tree2 = tree2.clone();
        move |_| match tree2.get_selected_items() {
            None => println!("Элемент не выбран!"),
            Some(vals) => print!(
                "Всего выбрано элементов: {}\nВыбранные элементы:\n{}",
                vals.len(), //.len - считаем длину
                vals.iter() //.iter - перебор
                    .map(|i| tree2.item_pathname(i).unwrap() + "\n") //Обращаемся к tree2, когда тыкаем на элемент; .unwrap - возвращаем объект
                    .collect::<String>()
            ),
        }
    });

    app.run().unwrap();
}