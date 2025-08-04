# -*- coding: utf-8 -*-
import gtk
import pango
import php
import os
import sys
reload(sys)
sys.setdefaultencoding('utf-8')
my = php.kit()
import myi18n
my18 = myi18n.kit()

PWD = os.path.dirname(os.path.realpath(sys.argv[0]))
# custom_dict_window.py
import json


CUSTOM_JSON_PATH = "%s\\custom.json" % PWD

class CustomDictWindow(object):   
    data = {}
    custom_UI = {
        "win": None,
        "entry_key": None,
        "text_words": None,
        "btn_save": None,
        "model": None
    }
    def __init__(self):        
        self.data = {}
        self.load_data()
        print(self)
        #print(parent);
        #self.win = parent
        #if self.win is None:
        self.custom_UI["win"] = gtk.Window(type=gtk.WINDOW_TOPLEVEL)
        self.custom_UI["win"].set_modal(True)
        self.custom_UI["win"].set_title(my18.auto("自定字詞功能"))
        self.custom_UI["win"].set_size_request(600, 400)
        self.custom_UI["win"].set_position(gtk.WIN_POS_CENTER)
        self.custom_UI["win"].connect("destroy", gtk.main_quit)

        vbox_main = gtk.VBox(False, 5)
        self.custom_UI["win"].add(vbox_main)

        # ===== 上方輸入區 =====
        hbox_input = gtk.HBox(False, 5)
        vbox_main.pack_start(hbox_input, False, False, 0)

        self.custom_UI["entry_key"] = gtk.Entry()
        self.custom_UI["entry_key"].set_max_length(5)
        hbox_input.pack_start(gtk.Label(my18.auto("字根：")), False, False, 0)
        hbox_input.pack_start(self.custom_UI["entry_key"], False, False, 0)

        self.custom_UI["text_words"] = gtk.TextView()
        self.custom_UI["text_words"].set_wrap_mode(gtk.WRAP_WORD)
        self.custom_UI["text_words"].set_size_request(300, 50)
        hbox_input.pack_start(gtk.Label(my18.auto("出字詞：")), False, False, 0)
        hbox_input.pack_start(self.custom_UI["text_words"], True, True, 0)

        self.btn_save = gtk.Button(my18.auto("儲存"))
        self.btn_save.connect("clicked", self.on_save_clicked)
        self.btn_save.set_size_request(60, 60)
        hbox_input.pack_start(self.btn_save, False, False, 0)

        # ===== 分隔線 =====
        separator = gtk.HSeparator()
        vbox_main.pack_start(separator, False, False, 5)

        # ===== 表格區 =====
        self.treeview = gtk.TreeView()
        self.custom_UI["model"] = gtk.ListStore(str, str, object)  # key, word string, original dict
        self.treeview.set_model(self.custom_UI["model"])

        for i, col_title in enumerate([my18.auto("字根"), my18.auto("字詞(點二下可以編修)")]):
            if i == 2:
                renderer = gtk.CellRendererText()
                column = gtk.TreeViewColumn(col_title, renderer, text=2)
            else:
                renderer = gtk.CellRendererText()
                column = gtk.TreeViewColumn(col_title, renderer, text=i)
            self.treeview.append_column(column)

        self.treeview.connect("row-activated", self.on_row_activated)

        scroll = gtk.ScrolledWindow()
        scroll.set_policy(gtk.POLICY_AUTOMATIC, gtk.POLICY_AUTOMATIC)
        scroll.add(self.treeview)
        vbox_main.pack_start(scroll, True, True, 0)

        self.refresh_table()
        self.custom_UI["win"].show_all()
        
        #return self.win

    def load_data(self):
        #print("Loading ... CUSTOM_JSON_PATH: %s " % (CUSTOM_JSON_PATH))
        if os.path.exists(CUSTOM_JSON_PATH):
            #with open(CUSTOM_JSON_PATH, "r", encoding="utf-8") as f:
            #    self.data = json.load(f)
            content = my.file_get_contents(CUSTOM_JSON_PATH)
            if content:
                try:
                    self.data = json.loads(content)
                except ValueError as e:
                    print("Error loading JSON data: %s" % e)
                    self.data = {}
            else:
                self.data = {}
        else:
            self.data = {}

    def save_data(self):
        #with open(CUSTOM_JSON_PATH, "w", encoding="utf-8") as f:
        #    json.dump(self.data, f, ensure_ascii=False, indent=2)
        my.file_put_contents(CUSTOM_JSON_PATH, my.json_encode_utf8(self.data), IS_APPEND=False)

    def refresh_table(self):
        self.custom_UI["model"].clear()
        for key, value in self.data.items():
            self.custom_UI["model"].append([key, value, my18.auto("編輯／刪除")]) 

    def on_save_clicked(self, widget):
        #print(self)
        #print(dir(self.self))
        #print(dir(self.entry_key))
        key = self.custom_UI["entry_key"].get_text()
        key = my.trim(key)
        #print(key)
        if key == "": # 如果沒有輸入字根，則不儲存
            return
        buf = self.custom_UI["text_words"].get_buffer()
        start, end = buf.get_bounds()
        value = buf.get_text(start, end) #.strip()
        if value == "": # 如果沒有輸入出字詞，則不儲存
            return
        self.data[key] = value
        self.save_data()
        self.refresh_table()
        self.custom_UI["entry_key"].set_text("")
        buf.set_text("")

    def on_row_activated(self, treeview, path, column):
        model = treeview.get_model()
        iter = model.get_iter(path)
        key = model.get_value(iter, 0)

        dialog = gtk.MessageDialog(
            #self.custom_UI["win"],
            None,
            gtk.DIALOG_MODAL,
            gtk.MESSAGE_QUESTION,
            gtk.BUTTONS_NONE,
            "%s%s%s" % (my18.auto("請選擇要對「"), key,my18.auto("」做什麼？"))
        )
        dialog.set_position(gtk.WIN_POS_CENTER_ALWAYS) # Always center the dialog
        dialog.add_button(my18.auto("編輯"), gtk.RESPONSE_YES)
        dialog.add_button(my18.auto("刪除"), gtk.RESPONSE_NO)
        dialog.add_button(my18.auto("取消"), gtk.RESPONSE_CANCEL)
        response = dialog.run()
        dialog.destroy()

        if response == gtk.RESPONSE_YES:
            self.custom_UI["entry_key"].set_text(key)
            buf = self.custom_UI["text_words"].get_buffer()
            buf.set_text(self.data[key])
        elif response == gtk.RESPONSE_NO:
            del self.data[key]
            self.save_data()
            self.refresh_table()


