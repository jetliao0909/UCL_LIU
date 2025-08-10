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
    
    custom_UI = {
        "parent": None, # 父視窗
        "data": {}, # 儲存自定字詞的資料
        "lastKey": None, # 編輯用
        "lastValue": None, # 編輯用
        "isEdit": False, # 編輯用
        "win": None,
        "entry_key": None,
        "text_words": None,
        "btn_save": None,
        "model": None
    }
    def __init__(self, parent=None):                
        self.load_data()
        #print(self)
        #print(parent);
        #self.win = parent
        #if self.win is None:
        self.custom_UI["parent"] = parent
        self.custom_UI["win"] = gtk.Window(type=gtk.WINDOW_TOPLEVEL)
        self.custom_UI["win"].set_modal(True)
        self.custom_UI["win"].set_title(my18.auto("自定字詞功能"))
        self.custom_UI["win"].set_size_request(600, 400)
        # 防最大化視窗大小
        self.custom_UI["win"].set_resizable(False)
        self.custom_UI["win"].set_position(gtk.WIN_POS_CENTER)
        self.custom_UI["win"].connect("destroy", gtk.main_quit)
        self.GLOBAL_FONT_FAMILY = "Mingliu,Serif,Malgun Gothic,roman" #roman
        vbox_main = gtk.VBox(False, 5)
        self.custom_UI["win"].add(vbox_main)
        vbox_main.set_border_width(5)

        # ===== 上方輸入區 =====
        hbox_input = gtk.HBox(False, 5)
        vbox_main.pack_start(hbox_input, False, False, 0)

        # ===== 輸入字根 =====
        self.custom_UI["entry_key"] = gtk.Entry()
        self.custom_UI["entry_key"].set_max_length(5)        
        self.custom_UI["entry_key"].modify_font(pango.FontDescription("Consolas bold 18"))
        self.custom_UI["entry_key"].set_size_request(120, 50)
        hbox_input.pack_start(gtk.Label(my18.auto("字根：")), False, False, 0)
        hbox_input.pack_start(self.custom_UI["entry_key"], False, False, 0)
        # 當 focus 進入 entry_key 時，如果是「肥模式」，切換回「英文模式」
        self.custom_UI["entry_key"].connect("focus-in-event", self.focusOn_entry_key)

        # ===== 輸入出字詞 =====
        self.custom_UI["text_words"] = gtk.TextView()
        self.custom_UI["text_words"].set_wrap_mode(gtk.WRAP_WORD)
        self.custom_UI["text_words"].set_size_request(300, 50)
        
        #self.custom_UI["text_words"].modify_font(pango.FontDescription("%s bold 12" % (self.GLOBAL_FONT_FAMILY)))
        self.custom_UI["text_words"].set_left_margin(5)
        self.custom_UI["text_words"].set_right_margin(5)        
        # border color #000
        self.custom_UI["text_words"].set_border_width(5)      
        self.custom_UI["text_words"].set_editable(True)
        self.custom_UI["text_words"].set_cursor_visible(True)
        #self.custom_UI["text_words"].modify_font(pango.FontDescription("%s bold 12" % (self.GLOBAL_FONT_FAMILY)))

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
        self.custom_UI["model"] = gtk.ListStore(int, str, str, object)  # step, key, word string, original dict
        self.treeview.set_model(self.custom_UI["model"])

        for i, col_title in enumerate([my18.auto("項次"), my18.auto("字根"), my18.auto("字詞(點二下可以編修)")]):
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
        # 視窗關閉時，主程式執行 load_word_root
        self.custom_UI["win"].connect("delete-event", self.on_window_delete_event)
    def on_window_delete_event(self, widget, event):
        #print("on_window_delete_event")
        #self.win.hide()
        self.custom_UI["win"].hide()
        #self.win.destroy()
        self.save_data()
        self.custom_UI["parent"].load_word_root() # 重新載入字根
    def focusOn_entry_key(self, widget, event):
        #print("focusOn_entry_key")
        # 如果是「肥模式」，切換回「英文模式」
        print("is_url: %s" % (self.custom_UI["parent"].is_ucl()))
        if self.custom_UI["parent"].is_ucl():
            self.custom_UI["parent"].toggle_ucl()  
            # 重新 focus 到 entry_key
            self.custom_UI["entry_key"].grab_focus()
        #return True # 返回 True 以防止事件繼續傳播
        

    def load_data(self):
        #print("Loading ... CUSTOM_JSON_PATH: %s " % (CUSTOM_JSON_PATH))
        if my.is_file(CUSTOM_JSON_PATH):            
            content = my.file_get_contents(CUSTOM_JSON_PATH)
            if content:
                try:
                    self.custom_UI["data"] = my.json_decode(content)
                    #print(self.custom_UI["data"])
                except ValueError as e:
                    print("Error loading JSON data: %s" % e)
                    self.custom_UI["data"] = {}
            else:
                self.custom_UI["data"] = {}
        else:
            self.custom_UI["data"] = {}

    def save_data(self):
        #with open(CUSTOM_JSON_PATH, "w", encoding="utf-8") as f:
        #    json.dump(self.custom_UI["data"], f, ensure_ascii=False, indent=2)
        my.file_put_contents(CUSTOM_JSON_PATH, my.json_encode_utf8(self.custom_UI["data"]), IS_APPEND=False)

    def refresh_table(self):
        self.custom_UI["model"].clear()
        for key in self.custom_UI["data"]:
            step = 1
            for value in self.custom_UI["data"][key]:
                #print("key: %s, value: %s" % (key, value))
                #print("type(value): %s" % (type(value)))                
                self.custom_UI["model"].append([step, key, value, my18.auto("編輯／刪除")]) 
                step += 1

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
        # 如果是編輯模式，則照舊的字根和出字詞更新
        if self.custom_UI["isEdit"]:
            #print("編輯模式")
            if self.custom_UI["lastKey"] is not None and self.custom_UI["lastValue"] is not None:
                # 刪除舊的字根和出字詞
                if self.custom_UI["lastKey"] in self.custom_UI["data"]:
                    if self.custom_UI["lastValue"] in self.custom_UI["data"][self.custom_UI["lastKey"]]:
                        self.custom_UI["data"][self.custom_UI["lastKey"]].remove(self.custom_UI["lastValue"])
                        if self.custom_UI["data"][self.custom_UI["lastKey"]] == []:
                            del self.custom_UI["data"][self.custom_UI["lastKey"]]
        # 結束編輯模式
        self.custom_UI["isEdit"] = False
        self.custom_UI["lastKey"] = None
        self.custom_UI["lastValue"] = None
                                                            
        # 如果字根已存在，則更新出字詞
        if key not in self.custom_UI["data"]:
            self.custom_UI["data"][key] = []
        # 如果出字詞已存在，則不重複添加
        if value in self.custom_UI["data"][key]:
            self.save_data()
            self.refresh_table()
            self.custom_UI["entry_key"].set_text("")
            buf.set_text("")
            return
        
        self.custom_UI["data"][key].append(value)
        #print(my.json_encode(self.custom_UI["data"]))
        self.save_data()
        self.refresh_table()
        self.custom_UI["entry_key"].set_text("")
        buf.set_text("")

    def on_row_activated(self, treeview, path, column):
        model = treeview.get_model()
        iter = model.get_iter(path)
        step = model.get_value(iter, 0)
        key = model.get_value(iter, 1)
        val = model.get_value(iter, 2)

        dialog = gtk.MessageDialog(
            #self.custom_UI["win"],
            None,
            gtk.DIALOG_MODAL,
            gtk.MESSAGE_QUESTION,
            gtk.BUTTONS_NONE,
            "%s%s%s %s" % (my18.auto("請選擇要對「"), key,val,my18.auto("」做什麼？"))
        )
        dialog.set_position(gtk.WIN_POS_CENTER_ALWAYS) # Always center the dialog

        # gtk.RESPONSE 順序左到右參考:
        # https://developer.gnome.org/gtk3/stable/gtk3-Dialog-Buttons.html#gtk-response-type-enum
        # 1. gtk.RESPONSE_HELP # 編輯
        # 2. gtk.RESPONSE_YES  # 刪除
        # 3. gtk.RESPONSE_NO   # 上移
        # 4. gtk.RESPONSE_APPLY # 下移
        # 5. gtk.RESPONSE_CANCEL # 取消
        # 6. gtk.RESPONSE_CLOSE
        

        dialog.add_button(my18.auto("編輯"), gtk.RESPONSE_HELP)
        dialog.add_button(my18.auto("刪除"), gtk.RESPONSE_YES)
        # 如果 data[key] 的長度大於 1 ，可以上移 或下移 調整排序
        if len(self.custom_UI["data"][key]) > 1 and step ==1:            
            dialog.add_button(my18.auto("下移"), gtk.RESPONSE_APPLY)
        elif len(self.custom_UI["data"][key]) > 1 and step == len(self.custom_UI["data"][key]):
            dialog.add_button(my18.auto("上移"), gtk.RESPONSE_NO)
        elif len(self.custom_UI["data"][key]) > 1 and step < len(self.custom_UI["data"][key]):
            dialog.add_button(my18.auto("上移"), gtk.RESPONSE_NO)
            dialog.add_button(my18.auto("下移"), gtk.RESPONSE_APPLY)
        else:
            # 如果只有一個出字詞，則不顯示上移或下移按鈕
            pass            

        dialog.add_button(my18.auto("取消"), gtk.RESPONSE_CANCEL)
        response = dialog.run()
        dialog.destroy()

        if response == gtk.RESPONSE_HELP:
            # 編輯模式
            self.custom_UI["lastKey"] = key
            self.custom_UI["lastValue"] = val
            self.custom_UI["isEdit"] = True

            self.custom_UI["entry_key"].set_text(key)
            buf = self.custom_UI["text_words"].get_buffer()
            buf.set_text(val)
        elif response == gtk.RESPONSE_YES:
            # 刪除
            for i in range(len(self.custom_UI["data"][key])):
                if self.custom_UI["data"][key][i] == val:
                    del self.custom_UI["data"][key][i]
                    break
            
            self.save_data()
            self.refresh_table()
        elif response == gtk.RESPONSE_NO:
            # 上移
            if step > 1:
                #print("上移")
                self.custom_UI["data"][key].insert(step - 2, self.custom_UI["data"][key].pop(step - 1))
                self.save_data()
                self.refresh_table()
        elif response == gtk.RESPONSE_APPLY:
            # 下移
            if step < len(self.custom_UI["data"][key]):
                #print("下移")
                self.custom_UI["data"][key].insert(step, self.custom_UI["data"][key].pop(step - 1))
                self.save_data()
                self.refresh_table()


