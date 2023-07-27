import os
from multiprocessing import cpu_count, Manager
from multiprocessing.pool import Pool
from queue import Queue, Empty
from contextlib import closing

import numpy as np
import py_dcm_finder_rs
from pydicom import dicomio, errors

import time

REQUIRED_TAGS = [
    'PatientID',
    'PatientName',
    'PatientBirthDate',
    'PatientSex',
    'StudyInstanceUID',
    'StudyDescription',
    'StudyDate',
    'SeriesInstanceUID',
    'SeriesDescription',
    'SOPInstanceUID',
    'Modality',
    'XRayTubeCurrent',
    'KVP',
    'InstanceNumber',
    'NumberOfFrames',
    'ImagePositionPatient',
]

DEFAULT_TAG_VAL = "*NO_TAG*"


def find_all_files(root_dir: str) -> list:
    file_list = []
    if os.path.exists(root_dir):
        for root, _, files in os.walk(root_dir):
            file_list += [os.path.join(root, f) for f in files]
    return file_list


def find_and_read_dcm_in_dir_multiproc(root_dir: str):
    # Находим все файлы по заданным в root_dir путям
    allfiles = find_all_files(root_dir)
    numfiles = len(allfiles)  # общее количество найденных файлов
    if not numfiles:
        return []
    # Если число процессов не задано, устанавливаем его равным числу ядер процессора
    num_procs = cpu_count()
    m = Manager()  # создаём менеджер процессов
    queue = m.Queue()  # очередь результатов поиска (<путь к Dicom-файлу>, <содержимое Dicom-файла>)
    res = []  # список процессов-обработчиков
    with closing(Pool(processes=num_procs)) as pool:
        for filesec in np.array_split(allfiles, num_procs):
            res.append(pool.apply_async(_load_dicom_files, (filesec, queue)))
        # повторяем, пока все процессы не завершили работу и очередь не пуста
        while any(not r.ready() for r in res) or not queue.empty():
            try:
                filename, dcm = queue.get(False)
            except Empty:  # Если очередь пуста, ждём завершения всех процессов
                pass


def _load_dicom_files(filenames: str, queue):
    """Для DICOM-файлов из списка filenames загружаем минимальный набор тегов и помещаем в очередь."""
    for filename in filenames:
        try:
            dcm = dicomio.read_file(filename, specific_tags=REQUIRED_TAGS, stop_before_pixels=True)
            tags = {t: dcm.get(t) if t in dcm else DEFAULT_TAG_VAL for t in REQUIRED_TAGS}
            queue.put((filename, tags))
        except errors.InvalidDicomError:
            pass
        except AttributeError:
            pass
        except:
            pass


def run():
    # Using only Python with one process
    start_time = time.time()
    paths = find_all_files(r"C:\Users\Alexandr\Desktop\test\1\Python in one process")
    for file_path in paths:
        if os.path.isdir(file_path):
            continue
        _dcm = dicomio.read_file(file_path, specific_tags=REQUIRED_TAGS, stop_before_pixels=True)
    py_long = time.time() - start_time
    # ----------------------------------------

    # Using Rust
    start_time = time.time()
    _ = py_dcm_finder_rs.load_dcm_files_in_dir(
        r"C:\Users\Alexandr\Desktop\test\1\Rust call from python",
        REQUIRED_TAGS, DEFAULT_TAG_VAL
    )
    rust_long = time.time() - start_time
    # ----------------------------------------

    # Using only Python with multiprocessing
    start_time = time.time()
    find_and_read_dcm_in_dir_multiproc(r"C:\Users\Alexandr\Desktop\test\1\Python multi process", )
    multi_py_long = time.time() - start_time
    # ----------------------------------------

    print(f"Всего файлов в папке: {len(paths)}")
    print("─" * 42)
    print(f"| {'Python single process':25} | {f'{py_long:.5f}':10} |")
    print(f"| {'Rust called from python':25} | {f'{rust_long:.5f}':10} |")
    print(f"| {'Python multiprocess.':25} | {f'{multi_py_long:.5f}':10} |")
    print("─" * 42)


if __name__ == '__main__':
    run()
